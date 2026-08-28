use std::convert::TryFrom;

use crate::backend::python::ast::node::{CoreOp, PythonCore};
use crate::backend::python::convert::builder::convert_builder;
use crate::backend::python::convert::call::convert_call;
use crate::backend::python::convert::class::convert_class;
use crate::backend::python::convert::common::convert_vec;
use crate::backend::python::convert::control_flow::{
    convert_cntrl_flow, scope_guarded, wrap_scoped,
};
use crate::backend::python::convert::definition::convert_def;
use crate::backend::python::convert::handle::convert_handle;
use crate::backend::python::convert::range_slice::convert_range_slice;
use crate::backend::python::convert::state::{Imports, State};
use crate::backend::python::name::ToPy;
use crate::backend::python::result::{GenResult, UnimplementedErr};
use crate::check::ast::NodeTy;
use crate::check::context::clss::concrete_to_python;
use crate::check::name::Name;
use crate::{ASTTy, Context};

mod builder;
mod call;
mod class;
mod common;
mod control_flow;
mod definition;
mod handle;
mod range_slice;

pub mod state;

pub fn convert_node(ast: &ASTTy, imp: &mut Imports, state: &State, ctx: &Context) -> GenResult {
    // Prevent these state properties from propagating further
    let must_assign_to = state.must_assign_to.clone();
    let is_last_must_be_ret = state.is_last_must_be_ret;

    let old_state = state.clone();
    let state = &state.must_assign_to(None, None).is_last_must_be_ret(false);

    let core = match &ast.node {
        NodeTy::Import {
            from,
            import,
            alias,
        } => PythonCore::Import {
            from: if let Some(from) = from {
                Some(Box::from(convert_node(from, imp, state, ctx)?))
            } else {
                None
            },
            import: convert_vec(import, imp, state, ctx)?,
            alias: convert_vec(alias, imp, state, ctx)?,
        },

        NodeTy::VariableDef { .. } | NodeTy::FunDef { .. } | NodeTy::FunArg { .. } => {
            convert_def(ast, imp, state, ctx)?
        }
        NodeTy::Reassign { left, right, op } => PythonCore::Assign {
            left: Box::from(match &left.node {
                NodeTy::FunctionCall { name, args, .. } if args.len() == 1 => PythonCore::Index {
                    item: Box::from(name.to_py(imp)),
                    range: Box::from(convert_node(&args[0], imp, state, ctx)?),
                },
                _ => convert_node(left, imp, state, ctx)?,
            }),
            right: Box::from(convert_node(right, imp, state, ctx)?),
            op: CoreOp::try_from((ast, op))?,
        },

        NodeTy::Block { statements } => PythonCore::Block {
            statements: convert_vec(statements, imp, state, ctx)?,
        },

        NodeTy::Int { lit } => PythonCore::Int { int: lit.clone() },
        NodeTy::Real { lit } => PythonCore::Float { float: lit.clone() },
        NodeTy::ENum { num, exp } => PythonCore::ENum {
            num: num.clone(),
            exp: if exp.is_empty() {
                String::from("0")
            } else {
                exp.clone()
            },
        },
        NodeTy::DocStr { lit } => PythonCore::DocStr {
            string: lit.clone(),
        },
        NodeTy::Str { lit, expressions } if expressions.is_empty() => PythonCore::Str {
            string: lit.clone(),
        },
        NodeTy::Str { lit, .. } => PythonCore::FStr {
            string: lit.clone(),
        },

        NodeTy::Undefined => PythonCore::None,
        NodeTy::ExpressionType { expr, .. } => {
            convert_node(expr, imp, &state.expand_ty(true), ctx)?
        }
        NodeTy::Id { lit } => PythonCore::Id {
            lit: concrete_to_python(lit),
        },
        NodeTy::Bool { lit } => PythonCore::Bool { boolean: *lit },

        NodeTy::Tuple { elements } if state.tup_lit => PythonCore::TupleLiteral {
            elements: convert_vec(elements, imp, state, ctx)?,
        },
        NodeTy::Tuple { elements } => PythonCore::Tuple {
            elements: convert_vec(elements, imp, state, ctx)?,
        },
        NodeTy::List { elements } => PythonCore::List {
            elements: convert_vec(elements, imp, state, ctx)?,
        },
        NodeTy::Dict { elements } => {
            let mut converted = vec![];
            for (from, to) in elements {
                let from = convert_node(from, imp, state, ctx)?;
                let to = convert_node(to, imp, state, ctx)?;
                converted.push((from, to));
            }
            PythonCore::Dictionary {
                elements: converted,
            }
        }
        NodeTy::Set { elements } => PythonCore::Set {
            elements: convert_vec(elements, imp, state, ctx)?,
        },
        NodeTy::Index { item, range } => PythonCore::Index {
            item: Box::from(convert_node(item, imp, state, ctx)?),
            range: Box::from(convert_node(range, imp, state, ctx)?),
        },

        NodeTy::DictBuilder { .. } => convert_builder(ast, imp, state, ctx)?,
        NodeTy::ListBuilder { .. } => convert_builder(ast, imp, state, ctx)?,
        NodeTy::SetBuilder { .. } => convert_builder(ast, imp, state, ctx)?,

        NodeTy::ReturnEmpty => PythonCore::Return {
            expr: Box::from(PythonCore::None),
        },
        NodeTy::Return { expr } if state.is_remove_last_ret => {
            convert_node(expr, imp, &state.remove_ret(false), ctx)?
        }
        NodeTy::Return { expr } => PythonCore::Return {
            expr: Box::from(convert_node(expr, imp, state, ctx)?),
        },

        NodeTy::IfElse { .. } => convert_cntrl_flow(ast, imp, &old_state, ctx)?,
        NodeTy::Match { .. } => convert_cntrl_flow(ast, imp, &old_state, ctx)?,
        NodeTy::While { .. } | NodeTy::For { .. } | NodeTy::Break | NodeTy::Continue => {
            convert_cntrl_flow(ast, imp, state, ctx)?
        }

        NodeTy::Not { expr } => PythonCore::Not {
            expr: Box::from(convert_node(expr, imp, state, ctx)?),
        },
        NodeTy::And { left, right } => PythonCore::And {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::Or { left, right } => PythonCore::Or {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::Eq { left, right } => PythonCore::Eq {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::Neq { left, right } => PythonCore::Neq {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::Add { left, right } => PythonCore::Add {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::Sub { left, right } => PythonCore::Sub {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::Mul { left, right } => PythonCore::Mul {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::Div { left, right } => PythonCore::Div {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::FDiv { left, right } => PythonCore::FDiv {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::Mod { left, right } => PythonCore::Mod {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::Pow { left, right } => PythonCore::Pow {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },

        NodeTy::AddU { expr } => PythonCore::AddU {
            expr: Box::from(convert_node(expr, imp, state, ctx)?),
        },
        NodeTy::SubU { expr } => PythonCore::SubU {
            expr: Box::from(convert_node(expr, imp, state, ctx)?),
        },
        NodeTy::Sqrt { expr } => {
            imp.add_import("math");
            PythonCore::Sqrt {
                expr: Box::from(convert_node(expr, imp, state, ctx)?),
            }
        }

        NodeTy::Le { left, right } => PythonCore::Le {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::Leq { left, right } => PythonCore::Leq {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::Ge { left, right } => PythonCore::Ge {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::Geq { left, right } => PythonCore::Geq {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },

        NodeTy::FunctionCall { .. } | NodeTy::PropertyCall { .. } => {
            convert_call(ast, imp, state, ctx)?
        }
        NodeTy::AnonFun { args, body } => PythonCore::AnonFun {
            args: convert_vec(args, imp, &state.expand_ty(false), ctx)?,
            body: Box::from(convert_node(body, imp, state, ctx)?),
        },

        NodeTy::In { left, right } => PythonCore::In {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::Range { .. } | NodeTy::Slice { .. } => convert_range_slice(ast, imp, state, ctx)?,

        NodeTy::Underscore => PythonCore::UnderScore,
        NodeTy::Question { left, right } => PythonCore::Or {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },

        NodeTy::Trait { .. } => convert_class(ast, imp, state, ctx)?,
        NodeTy::Class { .. } => convert_class(ast, imp, state, ctx)?,
        NodeTy::Parent { .. } => convert_class(ast, imp, state, ctx)?,

        NodeTy::Condition { .. } => return Err(Box::from(UnimplementedErr::new(ast, "condition"))),

        NodeTy::Using {
            resource,
            alias: Some((alias, ..)),
            expr,
        } => {
            let expr_core = convert_node(expr, imp, state, ctx)?;
            // The `as alias` binding is a fresh name too (like a for-loop's own control
            // variable), not just whatever `expr` itself directly `def`s -- guard it the same
            // way.
            let expr_core = match &alias.node {
                NodeTy::Id { lit } => wrap_scoped(std::slice::from_ref(lit), expr_core),
                _ => expr_core,
            };
            PythonCore::WithAs {
                resource: Box::from(convert_node(resource, imp, state, ctx)?),
                alias: Box::from(convert_node(alias, imp, &state.expand_ty(false), ctx)?),
                expr: Box::from(scope_guarded(expr, expr_core)),
            }
        }
        NodeTy::Using { resource, expr, .. } => {
            let expr_core = convert_node(expr, imp, state, ctx)?;
            PythonCore::With {
                resource: Box::from(convert_node(resource, imp, state, ctx)?),
                expr: Box::from(scope_guarded(expr, expr_core)),
            }
        }

        NodeTy::Raise { .. } | NodeTy::Handle { .. } => convert_handle(ast, imp, state, ctx)?,

        NodeTy::Pass => PythonCore::Pass,
        _ => PythonCore::Empty,
    };

    let core = if let Some((assign_to, name)) = must_assign_to {
        append_assign(&core, &assign_to, &name, imp)
    } else {
        core
    };

    let core = if is_last_must_be_ret {
        append_ret(&core)
    } else {
        core
    };

    Ok(core)
}

fn append_assign(
    core: &PythonCore,
    assign_to: &PythonCore,
    name: &Option<Name>,
    imp: &mut Imports,
) -> PythonCore {
    match &core {
        PythonCore::Block { ref statements } => match statements.last() {
            Some(last) => {
                let last = append_assign(last, assign_to, name, imp);
                let (mut statements, idx): (Vec<PythonCore>, usize) =
                    (statements.clone(), statements.len() - 1);
                statements[idx] = last;
                PythonCore::Block { statements }
            }
            None => core.clone(),
        },
        PythonCore::IfElse { cond, then, el } => PythonCore::IfElse {
            cond: cond.clone(),
            then: Box::from(append_assign(then, assign_to, name, imp)),
            el: Box::from(append_assign(el, assign_to, name, imp)),
        },
        PythonCore::Match { expr, cases } => PythonCore::Match {
            expr: expr.clone(),
            cases: cases
                .iter()
                .map(|c| append_assign(c, assign_to, name, imp))
                .collect(),
        },
        PythonCore::Case { expr, body } => PythonCore::Case {
            expr: expr.clone(),
            body: Box::from(append_assign(body, assign_to, name, imp)),
        },
        PythonCore::TryExcept {
            setup,
            attempt,
            except,
        } => PythonCore::TryExcept {
            setup: setup.clone(),
            attempt: Box::from(append_assign(attempt, assign_to, name, imp)),
            except: except
                .iter()
                .map(|e| append_assign(e, assign_to, name, imp))
                .collect(),
        },
        PythonCore::ExceptId { id, class, body } => PythonCore::ExceptId {
            id: id.clone(),
            class: class.clone(),
            body: Box::from(append_assign(body, assign_to, name, imp)),
        },
        PythonCore::Except { class, body } => PythonCore::Except {
            class: class.clone(),
            body: Box::from(append_assign(body, assign_to, name, imp)),
        },
        expr if skip_assign(expr) => core.clone(),
        _ => PythonCore::VarDef {
            var: Box::from(assign_to.clone()),
            ty: name.clone().map(|name| Box::from(name.to_py(imp))),
            expr: Option::from(Box::from(core.clone())),
        },
    }
}

fn append_ret(core: &PythonCore) -> PythonCore {
    match core {
        PythonCore::Block { ref statements } => match statements.last() {
            Some(last) => {
                let last = append_ret(last);
                let (mut statements, idx): (Vec<PythonCore>, usize) =
                    (statements.clone(), statements.len() - 1);
                statements[idx] = last;
                PythonCore::Block { statements }
            }
            None => PythonCore::Block {
                statements: vec![PythonCore::Return {
                    expr: Box::from(PythonCore::None),
                }],
            },
        },
        PythonCore::IfElse { cond, then, el } => PythonCore::IfElse {
            cond: cond.clone(),
            then: Box::from(append_ret(then)),
            el: Box::from(append_ret(el)),
        },
        PythonCore::Match { expr, cases } => PythonCore::Match {
            expr: expr.clone(),
            cases: cases.iter().map(append_ret).collect(),
        },
        PythonCore::Case { expr, body } => PythonCore::Case {
            expr: expr.clone(),
            body: Box::from(append_ret(body)),
        },
        PythonCore::TryExcept {
            setup,
            attempt,
            except,
        } => PythonCore::TryExcept {
            setup: setup.clone(),
            attempt: Box::from(append_ret(attempt)),
            except: except.iter().map(append_ret).collect(),
        },
        PythonCore::ExceptId { id, class, body } => PythonCore::ExceptId {
            id: id.clone(),
            class: class.clone(),
            body: Box::from(append_ret(body)),
        },
        PythonCore::Except { class, body } => PythonCore::Except {
            class: class.clone(),
            body: Box::from(append_ret(body)),
        },
        core if skip_return(core) => core.clone(),
        _ => PythonCore::Return {
            expr: Box::from(core.clone()),
        },
    }
}

fn skip_assign(core: &PythonCore) -> bool {
    skip_return(core) || matches!(core, PythonCore::VarDef { .. } | PythonCore::Assign { .. })
}

fn skip_return(core: &PythonCore) -> bool {
    matches!(core, PythonCore::Return { .. } | PythonCore::Raise { .. })
}

#[cfg(test)]
mod tests {
    use crate::backend::python::ast::node::{CoreOp, PythonCore};
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
    fn break_verify() {
        let _break = to_pos!(Node::Break);
        assert_eq!(gen(&ASTTy::from(&_break)).unwrap(), PythonCore::Break);
    }

    #[test]
    fn continue_verify() {
        let _continue = to_pos!(Node::Continue);
        assert_eq!(gen(&ASTTy::from(&_continue)).unwrap(), PythonCore::Continue);
    }

    #[test]
    fn pass_verify() {
        let pass = to_pos!(Node::Pass);
        assert_eq!(gen(&ASTTy::from(&pass)).unwrap(), PythonCore::Pass);
    }

    #[test]
    fn return_verify() {
        let expr = to_pos!(Node::Str {
            lit: String::from("a"),
            expressions: vec![]
        });
        let print_stmt = to_pos!(Node::Return { expr });

        assert_eq!(
            gen(&ASTTy::from(&print_stmt)).unwrap(),
            PythonCore::Return {
                expr: Box::from(PythonCore::Str {
                    string: String::from("a")
                })
            }
        );
    }

    #[test]
    fn return_empty_verify() {
        let print_stmt = to_pos!(Node::ReturnEmpty);
        assert_eq!(
            gen(&ASTTy::from(&print_stmt)).unwrap(),
            PythonCore::Return {
                expr: Box::from(PythonCore::None)
            }
        );
    }

    #[test]
    fn import_verify() {
        let _break = to_pos!(Node::Import {
            from: None,
            import: vec![to_pos_unboxed!(Node::Id {
                lit: String::from("a")
            })],
            alias: vec![to_pos_unboxed!(Node::Id {
                lit: String::from("b")
            })]
        });

        assert_eq!(
            gen(&ASTTy::from(&_break)).unwrap(),
            PythonCore::Import {
                from: None,
                import: vec![PythonCore::Id {
                    lit: String::from("a")
                }],
                alias: vec![PythonCore::Id {
                    lit: String::from("b")
                }],
            }
        );
    }

    macro_rules! verify {
        ($ast:ident) => {{
            let left = Node::Id {
                lit: String::from("left"),
            };
            let right = Node::Id {
                lit: String::from("right"),
            };
            let add_node = to_pos!(Node::$ast {
                left: to_pos!(left),
                right: to_pos!(right)
            });

            let (left, right) = match gen(&ASTTy::from(&add_node)) {
                Ok(PythonCore::$ast { left, right }) => (left, right),
                other => panic!("Expected binary operation but was {:?}", other),
            };

            assert_eq!(
                *left,
                PythonCore::Id {
                    lit: String::from("left")
                }
            );
            assert_eq!(
                *right,
                PythonCore::Id {
                    lit: String::from("right")
                }
            );
        }};
    }

    macro_rules! verify_unary {
        ($ast:ident) => {{
            let expr = to_pos!(Node::Id {
                lit: String::from("expression")
            });
            let add_node = to_pos!(Node::$ast { expr });

            let expr_des = match gen(&ASTTy::from(&add_node)) {
                Ok(PythonCore::$ast { expr }) => expr,
                other => panic!("Expected unary operation but was {:?}", other),
            };

            assert_eq!(
                *expr_des,
                PythonCore::Id {
                    lit: String::from("expression")
                }
            );
        }};
    }

    #[test]
    fn add_verify() {
        verify!(Add);
    }

    #[test]
    fn sub_verify() {
        verify!(Sub);
    }

    #[test]
    fn mul_verify() {
        verify!(Mul);
    }

    #[test]
    fn div_verify() {
        verify!(Div);
    }

    #[test]
    fn mod_verify() {
        verify!(Mod);
    }

    #[test]
    fn pow_verify() {
        verify!(Pow);
    }

    #[test]
    fn add_unary_verify() {
        verify_unary!(AddU);
    }

    #[test]
    fn sub_unary_verify() {
        verify_unary!(SubU);
    }

    #[test]
    fn sqrt_verify() {
        let expr = to_pos!(Node::Id {
            lit: String::from("expression")
        });
        let add_node = to_pos!(Node::Sqrt { expr });

        let (import, expr_des) = match gen(&ASTTy::from(&add_node)) {
            Ok(PythonCore::Block { statements }) => (statements[0].clone(), statements[1].clone()),
            other => panic!("Expected unary operation but was {other:?}"),
        };

        assert_eq!(
            import,
            PythonCore::Import {
                from: None,
                import: vec![PythonCore::Id {
                    lit: String::from("math")
                }],
                alias: vec![],
            }
        );
        assert_eq!(
            expr_des,
            PythonCore::Sqrt {
                expr: Box::from(PythonCore::Id {
                    lit: String::from("expression")
                })
            }
        );
    }

    #[test]
    fn le_verify() {
        verify!(Le);
    }

    #[test]
    fn leq_verify() {
        verify!(Leq);
    }

    #[test]
    fn ge_verify() {
        verify!(Ge);
    }

    #[test]
    fn geq_verify() {
        verify!(Geq);
    }

    #[test]
    fn neq_verify() {
        verify!(Neq);
    }

    #[test]
    fn not_verify() {
        verify_unary!(Not);
    }

    #[test]
    fn and_verify() {
        verify!(And);
    }

    #[test]
    fn or_verify() {
        verify!(Or);
    }

    #[test]
    fn tuple_verify() {
        let elements = vec![
            to_pos_unboxed!(Node::ENum {
                num: String::from("a"),
                exp: String::from("100")
            }),
            to_pos_unboxed!(Node::Real {
                lit: String::from("3000.5")
            }),
        ];
        let tuple = to_pos!(Node::Tuple { elements });
        let core = gen(&ASTTy::from(&tuple));

        let core_elements = match core {
            Ok(PythonCore::Tuple { elements }) => elements,
            other => panic!("Expected tuple but got {other:?}"),
        };

        assert_eq!(
            core_elements[0],
            PythonCore::ENum {
                num: String::from("a"),
                exp: String::from("100")
            }
        );
        assert_eq!(
            core_elements[1],
            PythonCore::Float {
                float: String::from("3000.5")
            }
        );
    }

    #[test]
    fn set_verify() {
        let elements = vec![
            to_pos_unboxed!(Node::Id {
                lit: String::from("a")
            }),
            to_pos_unboxed!(Node::Id {
                lit: "True".to_string()
            }),
        ];
        let set = to_pos!(Node::Set { elements });
        let core = gen(&ASTTy::from(&set));

        let core_elements = match core {
            Ok(PythonCore::Set { elements }) => elements,
            other => panic!("Expected set but got {other:?}"),
        };

        assert_eq!(
            core_elements[0],
            PythonCore::Id {
                lit: String::from("a")
            }
        );
        assert_eq!(core_elements[1], PythonCore::Bool { boolean: true });
    }

    #[test]
    fn list_verify() {
        let elements = vec![
            to_pos_unboxed!(Node::ENum {
                num: String::from("a"),
                exp: String::from("100")
            }),
            to_pos_unboxed!(Node::Real {
                lit: String::from("3000.5")
            }),
        ];
        let tuple = to_pos!(Node::List { elements });
        let core = gen(&ASTTy::from(&tuple));

        let core_elements = match core {
            Ok(PythonCore::List { elements }) => elements,
            other => panic!("Expected tuple but got {other:?}"),
        };

        assert_eq!(
            core_elements[0],
            PythonCore::ENum {
                num: String::from("a"),
                exp: String::from("100")
            }
        );
        assert_eq!(
            core_elements[1],
            PythonCore::Float {
                float: String::from("3000.5")
            }
        );
    }

    #[test]
    fn set_builder_verify() {
        let item = to_pos!(Node::Id {
            lit: String::from("a")
        });
        let conditions = vec![];
        let list_builder = to_pos!(Node::SetBuilder { item, conditions });

        let desugar_result = gen(&ASTTy::from(&list_builder));
        assert!(desugar_result.is_err());
    }

    #[test]
    fn list_builder_verify() {
        let item = to_pos!(Node::Id {
            lit: String::from("a")
        });
        let conditions = vec![];
        let list_builder = to_pos!(Node::ListBuilder { item, conditions });

        let desugar_result = gen(&ASTTy::from(&list_builder));
        assert!(desugar_result.is_err());
    }

    #[test]
    fn using_verify() {
        let resource = to_pos!(Node::Id {
            lit: String::from("my_resource")
        });
        let alias = Some((
            to_pos!(Node::Id {
                lit: String::from("other")
            }),
            false,
            None,
        ));
        let expr = to_pos!(Node::Int {
            lit: String::from("9")
        });
        let using = to_pos!(Node::Using {
            resource,
            alias,
            expr
        });

        let Ok(PythonCore::WithAs {
            resource,
            alias,
            expr,
        }) = gen(&ASTTy::from(&using))
        else {
            panic!("Expected with as but was {:?}", gen(&ASTTy::from(&using)))
        };

        assert_eq!(
            *resource,
            PythonCore::Id {
                lit: String::from("my_resource")
            }
        );
        assert_eq!(
            *alias,
            PythonCore::Id {
                lit: String::from("other")
            }
        );
        // `alias` (`other`) is a fresh binding for the `with` block's own scope, so `expr` is
        // wrapped the same way a shadowing `def` would be -- see `scope_guarded`/`wrap_scoped`.
        let PythonCore::Block { statements } = *expr else {
            panic!("Expected a scope-guarded with-as expr, was {expr:?}");
        };
        assert_eq!(
            statements.as_slice(),
            &[
                PythonCore::VarDef {
                    var: Box::from(PythonCore::Id {
                        lit: String::from("__mamba_other_existed")
                    }),
                    ty: None,
                    expr: Some(Box::from(PythonCore::In {
                        left: Box::from(PythonCore::Str {
                            string: String::from("other")
                        }),
                        right: Box::from(PythonCore::FunctionCall {
                            function: Box::from(PythonCore::Id {
                                lit: String::from("locals")
                            }),
                            args: vec![],
                        }),
                    })),
                },
                PythonCore::VarDef {
                    var: Box::from(PythonCore::Id {
                        lit: String::from("__mamba_other_saved")
                    }),
                    ty: None,
                    expr: Some(Box::from(PythonCore::Ternary {
                        cond: Box::from(PythonCore::Id {
                            lit: String::from("__mamba_other_existed")
                        }),
                        then: Box::from(PythonCore::Id {
                            lit: String::from("other")
                        }),
                        el: Box::from(PythonCore::None),
                    })),
                },
                PythonCore::Int {
                    int: String::from("9")
                },
                PythonCore::IfElse {
                    cond: Box::from(PythonCore::Id {
                        lit: String::from("__mamba_other_existed")
                    }),
                    then: Box::from(PythonCore::Assign {
                        left: Box::from(PythonCore::Id {
                            lit: String::from("other")
                        }),
                        right: Box::from(PythonCore::Id {
                            lit: String::from("__mamba_other_saved")
                        }),
                        op: CoreOp::Assign,
                    }),
                    el: Box::from(PythonCore::Del {
                        name: String::from("other")
                    }),
                },
            ]
        );
    }

    #[test]
    fn using_no_as_verify() {
        let resource = to_pos!(Node::Id {
            lit: String::from("other")
        });
        let expr = to_pos!(Node::Int {
            lit: String::from("2341")
        });
        let using = to_pos!(Node::Using {
            resource,
            alias: None,
            expr
        });

        let (resource, expr) = match gen(&ASTTy::from(&using)) {
            Ok(PythonCore::With { resource, expr }) => (resource, expr),
            other => panic!("Expected with but was {other:?}"),
        };

        assert_eq!(
            *resource,
            PythonCore::Id {
                lit: String::from("other")
            }
        );
        assert_eq!(
            *expr,
            PythonCore::Int {
                int: String::from("2341")
            }
        );
    }

    #[test]
    fn handle_empty_verify() {
        let expr_or_stmt = to_pos!(Node::Id {
            lit: String::from("my_fun")
        });
        let handle = to_pos!(Node::Handle {
            expr_or_stmt,
            cases: vec![]
        });

        let (setup, _try, except) = match gen(&ASTTy::from(&handle)) {
            Ok(PythonCore::TryExcept {
                setup,
                attempt,
                except,
            }) => (setup.clone(), attempt.clone(), except.clone()),
            other => panic!("Expected try except but was {other:?}"),
        };

        assert_eq!(setup, None);
        assert_eq!(
            *_try,
            PythonCore::Id {
                lit: String::from("my_fun")
            }
        );
        assert!(except.is_empty());
    }

    #[test]
    fn handle_verify() {
        let expr_or_stmt = to_pos!(Node::Id {
            lit: String::from("my_fun")
        });
        let cond = to_pos!(Node::ExpressionType {
            expr: to_pos!(Node::Id {
                lit: String::from("err")
            }),
            mutable: false,
            ty: Some(to_pos!(Node::Type {
                id: to_pos!(Node::Id {
                    lit: String::from("my_type")
                }),
                generics: vec![]
            }))
        });
        let body = to_pos!(Node::Int {
            lit: String::from("9999")
        });
        let case = to_pos_unboxed!(Node::Case { cond, body });
        let handle = to_pos!(Node::Handle {
            expr_or_stmt,
            cases: vec![case]
        });

        let Ok(PythonCore::TryExcept {
            setup,
            attempt,
            except,
        }) = gen(&ASTTy::from(&handle))
        else {
            panic!(
                "Expected try except but was {:?}",
                gen(&ASTTy::from(&handle))
            )
        };

        assert_eq!(setup, None);
        assert_eq!(
            *attempt,
            PythonCore::Id {
                lit: String::from("my_fun")
            }
        );
        assert_eq!(except.len(), 1);
        let PythonCore::ExceptId { id, class, body } = &except[0] else {
            panic!("Expected except case but was {:?}", except[0])
        };

        assert_eq!(
            *id,
            Box::from(PythonCore::Id {
                lit: String::from("err")
            })
        );
        assert_eq!(
            *class,
            Box::from(PythonCore::Type {
                lit: String::from("my_type"),
                generics: vec![]
            })
        );
        assert_eq!(
            *body,
            Box::from(PythonCore::Int {
                int: String::from("9999")
            })
        );
    }
}
