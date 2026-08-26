use crate::backend::python::ast::node::{CoreOp, PythonCore};
use crate::backend::python::convert::convert_node;
use crate::backend::python::convert::state::{Imports, State};
use crate::backend::python::result::{GenResult, UnimplementedErr};
use crate::check::ast::NodeTy;
use crate::{ASTTy, Context};

pub fn convert_cntrl_flow(
    ast: &ASTTy,
    imp: &mut Imports,
    state: &State,
    ctx: &Context,
) -> GenResult {
    Ok(match &ast.node {
        NodeTy::IfElse { cond, then, el } => {
            let cond = Box::from(convert_node(
                cond,
                imp,
                &state.is_last_must_be_ret(false).must_assign_to(None, None),
                ctx,
            )?);

            match el {
                Some(el) => {
                    if !state.is_last_must_be_ret
                        && ast.ty.is_some()
                        && is_valid_in_ternary(then, el)
                    {
                        let state = state
                            .is_last_must_be_ret(false)
                            .remove_ret(true)
                            .must_assign_to(None, None);

                        // A ternary's arms are expressions, not statements -- `is_valid_in_ternary`
                        // already rules out anything (a `Block`, in particular) that could contain
                        // a `def`, so there's nothing here that needs scope-guarding.
                        PythonCore::Ternary {
                            cond,
                            then: Box::from(convert_node(then, imp, &state, ctx)?),
                            el: Box::from(convert_node(el, imp, &state, ctx)?),
                        }
                    } else {
                        let then_core = convert_node(then, imp, state, ctx)?;
                        let el_core = convert_node(el, imp, state, ctx)?;
                        PythonCore::IfElse {
                            cond,
                            then: Box::from(scope_guarded(then, then_core)),
                            el: Box::from(scope_guarded(el, el_core)),
                        }
                    }
                }
                None => {
                    let then_core = convert_node(then, imp, state, ctx)?;
                    PythonCore::If {
                        cond,
                        then: Box::from(scope_guarded(then, then_core)),
                    }
                }
            }
        }
        NodeTy::Match {
            cond,
            cases: match_cases,
        } => {
            let expr = Box::from(convert_node(
                cond,
                imp,
                &state.is_last_must_be_ret(false).must_assign_to(None, None),
                ctx,
            )?);

            let mut cases = vec![];
            for case in match_cases {
                if let NodeTy::Case { cond, body } = &case.node {
                    if let NodeTy::ExpressionType { expr, .. } = &cond.node {
                        let body_core = convert_node(body.as_ref(), imp, state, ctx)?;
                        cases.push(PythonCore::Case {
                            expr: Box::from(convert_node(
                                expr.as_ref(),
                                imp,
                                &state.is_last_must_be_ret(false).must_assign_to(None, None),
                                ctx,
                            )?),
                            body: Box::from(scope_guarded(body.as_ref(), body_core)),
                        })
                    }
                }
            }

            PythonCore::Match { expr, cases }
        }
        NodeTy::While { cond, body } => {
            let while_core = PythonCore::While {
                cond: Box::from(convert_node(cond, imp, state, ctx)?),
                body: Box::from(convert_node(body, imp, state, ctx)?),
            };
            // Unlike an `if`/`case` branch, `body` runs every iteration -- the guard has to
            // wrap the *whole loop* (set up once before it, torn down once after), not `body`
            // itself, or a second iteration would "save" the value the first iteration's own
            // shadowing def already left behind, not the real outer one.
            wrap_scoped(&direct_def_names(body), while_core)
        }
        NodeTy::For { expr, col, body } => {
            let for_core = PythonCore::For {
                expr: Box::from(convert_node(expr, imp, state, ctx)?),
                col: Box::from(convert_node(col, imp, state, ctx)?),
                body: Box::from(convert_node(body, imp, state, ctx)?),
            };
            // The loop variable itself is always a fresh binding to guard, same reasoning as
            // `While` above for why the whole loop (not just `body`) gets wrapped.
            let mut names = vec![];
            if let NodeTy::Id { lit } = &expr.node {
                names.push(lit.clone());
            }
            for name in direct_def_names(body) {
                if !names.contains(&name) {
                    names.push(name);
                }
            }
            wrap_scoped(&names, for_core)
        }
        NodeTy::Break => PythonCore::Break,
        NodeTy::Continue => PythonCore::Continue,
        other => {
            let msg = format!("Expected control flow but was: {other:?}.");
            return Err(Box::from(UnimplementedErr::new(ast, &msg)));
        }
    })
}

/// Wrap `core` (the already-converted form of `body`) so any `def` directly inside `body` (not
/// nested deeper -- a `def` inside a further-nested block is that block's own responsibility,
/// handled when *it* is converted) doesn't leak or overwrite an outer variable of the same name
/// once this block exits. `body` runs at most once here (an `if`/`case` branch), so the guard can
/// wrap `core` itself directly -- contrast [`wrap_scoped`], used where the body is a loop.
pub(super) fn scope_guarded(body: &ASTTy, core: PythonCore) -> PythonCore {
    wrap_scoped(&direct_def_names(body), core)
}

/// The names `body` directly `def`s at its own top level -- i.e. not inside a further-nested
/// block. `body` is either a single statement or a `NodeTy::Block` of statements, matching how
/// Mamba represents a block/branch body.
fn direct_def_names(body: &ASTTy) -> Vec<String> {
    let statements: Vec<&ASTTy> = match &body.node {
        NodeTy::Block { statements } => statements.iter().collect(),
        _ => vec![body],
    };

    let mut names = vec![];
    for statement in statements {
        if let NodeTy::VariableDef { var: id_ast_ty, .. } = &statement.node {
            if let NodeTy::Id { lit } = &id_ast_ty.node {
                if !names.contains(lit) {
                    names.push(lit.clone());
                }
            }
        }
    }
    names
}

/// Sandwich `core` between a runtime existed/saved setup and a restore-or-delete teardown, one
/// pair per name in `names` -- so each of those names' bindings, as `core` leaves them, never
/// escape to whatever called this. A no-op (`core` unchanged) when `names` is empty.
///
/// This is the whole mechanism behind Mamba having real block scoping for `def`, unlike Python
/// (which this backend must still *behave* like Python for everything else -- e.g. reassigning
/// an outer variable with `:=`, which isn't a new binding, is untouched by this and keeps working
/// exactly as before; only fresh bindings introduced by `def` are undone here).
///
/// Whether a name was already bound has to be decided at runtime (via `locals()`), since nothing
/// has kept scope information around by the time code generation runs:
///
/// ```python
/// __mamba_i_existed = 'i' in locals()
/// __mamba_i_saved = i if __mamba_i_existed else None
/// <core>
/// if __mamba_i_existed:
///     i = __mamba_i_saved
/// else:
///     del i
/// ```
pub(super) fn wrap_scoped(names: &[String], core: PythonCore) -> PythonCore {
    if names.is_empty() {
        return core;
    }

    let mut statements: Vec<PythonCore> = names.iter().flat_map(|n| setup(n)).collect();
    // Flatten rather than nest `core` as a single list item: `to_py`'s renderer indents each of
    // a `Block`'s *items* uniformly, but a `Block` item that is itself a `Block` also indents its
    // own first line, doubling up on just that one line. Splicing its statements in directly
    // keeps every item here a plain (non-`Block`) statement, so they all render at the same,
    // correct indent.
    match core {
        PythonCore::Block { statements: inner } => statements.extend(inner),
        other => statements.push(other),
    }
    statements.extend(names.iter().map(|n| restore_or_delete(n)));

    PythonCore::Block { statements }
}

fn id(lit: &str) -> PythonCore {
    PythonCore::Id {
        lit: String::from(lit),
    }
}

fn setup(name: &str) -> [PythonCore; 2] {
    let existed = format!("__mamba_{name}_existed");
    let saved = format!("__mamba_{name}_saved");
    [
        PythonCore::VarDef {
            var: Box::from(id(&existed)),
            ty: None,
            expr: Some(Box::from(PythonCore::In {
                left: Box::from(PythonCore::Str {
                    string: String::from(name),
                }),
                right: Box::from(PythonCore::FunctionCall {
                    function: Box::from(id("locals")),
                    args: vec![],
                }),
            })),
        },
        PythonCore::VarDef {
            var: Box::from(id(&saved)),
            ty: None,
            expr: Some(Box::from(PythonCore::Ternary {
                cond: Box::from(id(&existed)),
                then: Box::from(id(name)),
                el: Box::from(PythonCore::None),
            })),
        },
    ]
}

fn restore_or_delete(name: &str) -> PythonCore {
    let existed = format!("__mamba_{name}_existed");
    let saved = format!("__mamba_{name}_saved");
    PythonCore::IfElse {
        cond: Box::from(id(&existed)),
        then: Box::from(PythonCore::Assign {
            left: Box::from(id(name)),
            right: Box::from(id(&saved)),
            op: CoreOp::Assign,
        }),
        el: Box::from(PythonCore::Del {
            name: String::from(name),
        }),
    }
}

fn is_valid_in_ternary(then: &ASTTy, el: &ASTTy) -> bool {
    is_expr_valid_in_ternary(then) && is_expr_valid_in_ternary(el)
}

/// Whether `node` can be rendered as a Python expression (needed for it to
/// appear as an arm of a ternary). `Block`/`Raise`/`Match`/`Handle` only have
/// statement forms in generated Python, and a nested `IfElse` is only OK if
/// both of *its* arms are themselves expressible (recurse, since the do/end
/// grammar no longer wraps single-statement arms in a `Block`, so a nested
/// `IfElse` whose arm is a multi-statement block is otherwise invisible here).
fn is_expr_valid_in_ternary(node: &ASTTy) -> bool {
    match &node.node {
        NodeTy::Block { .. }
        | NodeTy::Raise { .. }
        | NodeTy::Match { .. }
        | NodeTy::Handle { .. } => false,
        NodeTy::IfElse { then, el, .. } => {
            el.as_ref().is_some_and(|el| is_valid_in_ternary(then, el))
        }
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use crate::backend::python::ast::node::PythonCore;
    use crate::backend::python::gen;
    use crate::common::position::Position;
    use crate::parse::ast::Node;
    use crate::parse::ast::AST;
    use crate::ASTTy;

    macro_rules! to_pos_unboxed {
        ($node:expr) => {{
            AST {
                pos: Position::invisible(),
                node: $node,
            }
        }};
    }

    macro_rules! to_pos {
        ($node:expr) => {{
            Box::from(to_pos_unboxed!($node))
        }};
    }

    #[test]
    fn if_verify() {
        let cond = to_pos!(Node::Id {
            lit: String::from("cond")
        });
        let then = to_pos!(Node::Id {
            lit: String::from("then")
        });
        let if_stmt = to_pos!(Node::IfElse {
            cond,
            then,
            el: None
        });

        let (core_cond, core_then) = match gen(&ASTTy::from(&if_stmt)) {
            Ok(PythonCore::If { cond, then }) => (cond, then),
            other => panic!("Expected reassign but was {other:?}"),
        };

        assert_eq!(
            *core_cond,
            PythonCore::Id {
                lit: String::from("cond")
            }
        );
        assert_eq!(
            *core_then,
            PythonCore::Id {
                lit: String::from("then")
            }
        );
    }

    #[test]
    fn if_else_verify() {
        let cond = to_pos!(Node::Id {
            lit: String::from("cond")
        });
        let then = to_pos!(Node::Id {
            lit: String::from("then")
        });
        let el = to_pos!(Node::Id {
            lit: String::from("else")
        });
        let if_stmt = to_pos!(Node::IfElse {
            cond,
            then,
            el: Some(el)
        });

        let (core_cond, core_then, core_else) = match gen(&ASTTy::from(&if_stmt)) {
            Ok(PythonCore::IfElse { cond, then, el }) => (cond, then, el),
            other => panic!("Expected reassign but was {other:?}"),
        };

        assert_eq!(
            *core_cond,
            PythonCore::Id {
                lit: String::from("cond")
            }
        );
        assert_eq!(
            *core_then,
            PythonCore::Id {
                lit: String::from("then")
            }
        );
        assert_eq!(
            *core_else,
            PythonCore::Id {
                lit: String::from("else")
            }
        );
    }

    #[test]
    fn while_verify() {
        let cond = to_pos!(Node::Id {
            lit: String::from("cond")
        });
        let body = to_pos!(Node::ENum {
            num: String::from("num"),
            exp: String::from("")
        });
        let while_stmt = to_pos!(Node::While { cond, body });

        let (core_cond, core_body) = match gen(&ASTTy::from(&while_stmt)) {
            Ok(PythonCore::While { cond, body }) => (cond, body),
            other => panic!("Expected reassign but was {other:?}"),
        };

        assert_eq!(
            *core_cond,
            PythonCore::Id {
                lit: String::from("cond")
            }
        );
        assert_eq!(
            *core_body,
            PythonCore::ENum {
                num: String::from("num"),
                exp: String::from("0")
            }
        );
    }

    #[test]
    fn for_verify() {
        let expr = to_pos!(Node::Id {
            lit: String::from("expr_1")
        });
        let col = to_pos!(Node::Id {
            lit: String::from("col")
        });
        let body = to_pos!(Node::Id {
            lit: String::from("body")
        });
        let for_stmt = to_pos!(Node::For { expr, col, body });

        // Wrapped in a scope-guarding `Block` (see `scope_guarded_for`) -- the `For` itself is
        // the third statement, sandwiched between the setup and the restore-or-delete.
        let (core_expr, core_col, core_body) = match gen(&ASTTy::from(&for_stmt)) {
            Ok(PythonCore::Block { statements }) => match statements.as_slice() {
                [_, _, PythonCore::For { expr, col, body }, _] => {
                    (expr.clone(), col.clone(), body.clone())
                }
                other => panic!("Expected a 4-statement scope-guarded for, was {other:?}"),
            },
            other => panic!("Expected for but was {other:?}"),
        };

        assert_eq!(
            *core_expr,
            PythonCore::Id {
                lit: String::from("expr_1")
            }
        );
        assert_eq!(
            *core_col,
            PythonCore::Id {
                lit: String::from("col")
            }
        );
        assert_eq!(
            *core_body,
            PythonCore::Id {
                lit: String::from("body")
            }
        );
    }

    #[test]
    fn range_verify() {
        let from = to_pos!(Node::Id {
            lit: String::from("a")
        });
        let to = to_pos!(Node::Id {
            lit: String::from("b")
        });
        let range = to_pos!(Node::Range {
            from,
            to,
            inclusive: false,
            step: None
        });

        let (from, to, step) = match gen(&ASTTy::from(&range)) {
            Ok(PythonCore::FunctionCall { function, args }) => {
                assert_eq!(
                    *function,
                    PythonCore::Id {
                        lit: String::from("range")
                    }
                );
                (args[0].clone(), args[1].clone(), args[2].clone())
            }
            other => panic!("Expected range but was {other:?}"),
        };

        assert_eq!(
            from,
            PythonCore::Id {
                lit: String::from("a")
            }
        );
        assert_eq!(
            to,
            PythonCore::Id {
                lit: String::from("b")
            }
        );
        assert_eq!(
            step,
            PythonCore::Int {
                int: String::from("1")
            }
        );
    }

    #[test]
    fn range_incl_verify() {
        let from = to_pos!(Node::Id {
            lit: String::from("a")
        });
        let to = to_pos!(Node::Id {
            lit: String::from("b")
        });
        let range = to_pos!(Node::Range {
            from,
            to,
            inclusive: true,
            step: None
        });

        let (from, to, step) = match gen(&ASTTy::from(&range)) {
            Ok(PythonCore::FunctionCall { function, args }) => {
                assert_eq!(
                    *function,
                    PythonCore::Id {
                        lit: String::from("range")
                    }
                );
                (args[0].clone(), args[1].clone(), args[2].clone())
            }
            other => panic!("Expected range but was {other:?}"),
        };

        assert_eq!(
            from,
            PythonCore::Id {
                lit: String::from("a")
            }
        );
        assert_eq!(
            to,
            PythonCore::Add {
                left: Box::from(PythonCore::Id {
                    lit: String::from("b")
                }),
                right: Box::from(PythonCore::Int {
                    int: String::from("1")
                }),
            }
        );
        assert_eq!(
            step,
            PythonCore::Int {
                int: String::from("1")
            }
        );
    }

    #[test]
    fn range_step_verify() {
        let from = to_pos!(Node::Id {
            lit: String::from("a")
        });
        let to = to_pos!(Node::Id {
            lit: String::from("b")
        });
        let step = Some(to_pos!(Node::Id {
            lit: String::from("c")
        }));
        let range = to_pos!(Node::Range {
            from,
            to,
            inclusive: false,
            step
        });

        let (from, to, step) = match gen(&ASTTy::from(&range)) {
            Ok(PythonCore::FunctionCall { function, args }) => {
                assert_eq!(
                    *function,
                    PythonCore::Id {
                        lit: String::from("range")
                    }
                );
                (args[0].clone(), args[1].clone(), args[2].clone())
            }
            other => panic!("Expected range but was {other:?}"),
        };

        assert_eq!(
            from,
            PythonCore::Id {
                lit: String::from("a")
            }
        );
        assert_eq!(
            to,
            PythonCore::Id {
                lit: String::from("b")
            }
        );
        assert_eq!(
            step,
            PythonCore::Id {
                lit: String::from("c")
            }
        );
    }
}
