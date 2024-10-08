use crate::check::ast::NodeTy;
use crate::generate::ast::node::Core;
use crate::generate::convert::convert_node;
use crate::generate::convert::state::{Imports, State};
use crate::generate::result::{GenResult, UnimplementedErr};
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
                    if ast.ty.is_some() && is_valid_in_ternary(then, el) {
                        let state = state
                            .is_last_must_be_ret(false)
                            .remove_ret(true)
                            .must_assign_to(None, None);

                        Core::Ternary {
                            cond,
                            then: Box::from(convert_node(then, imp, &state, ctx)?),
                            el: Box::from(convert_node(el, imp, &state, ctx)?),
                        }
                    } else {
                        Core::IfElse {
                            cond,
                            then: Box::from(convert_node(then, imp, state, ctx)?),
                            el: Box::from(convert_node(el, imp, state, ctx)?),
                        }
                    }
                }
                None => Core::If {
                    cond,
                    then: Box::from(convert_node(then, imp, state, ctx)?),
                },
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
                        cases.push(Core::Case {
                            expr: Box::from(convert_node(
                                expr.as_ref(),
                                imp,
                                &state.is_last_must_be_ret(false).must_assign_to(None, None),
                                ctx,
                            )?),
                            body: Box::from(convert_node(body.as_ref(), imp, state, ctx)?),
                        })
                    }
                }
            }

            Core::Match { expr, cases }
        }
        NodeTy::While { cond, body } => Core::While {
            cond: Box::from(convert_node(cond, imp, state, ctx)?),
            body: Box::from(convert_node(body, imp, state, ctx)?),
        },
        NodeTy::For { expr, col, body } => Core::For {
            expr: Box::from(convert_node(expr, imp, state, ctx)?),
            col: Box::from(convert_node(col, imp, state, ctx)?),
            body: Box::from(convert_node(body, imp, state, ctx)?),
        },
        NodeTy::Break => Core::Break,
        NodeTy::Continue => Core::Continue,
        other => {
            let msg = format!("Expected control flow but was: {other:?}.");
            return Err(Box::from(UnimplementedErr::new(ast, &msg)));
        }
    })
}

fn is_valid_in_ternary(then: &ASTTy, el: &ASTTy) -> bool {
    !matches!(then.node, NodeTy::Block { .. } | NodeTy::Raise { .. })
        && !matches!(el.node, NodeTy::Block { .. } | NodeTy::Raise { .. })
}

#[cfg(test)]
mod tests {
    use crate::common::position::Position;
    use crate::generate::ast::node::Core;
    use crate::generate::gen;
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
            Ok(Core::If { cond, then }) => (cond, then),
            other => panic!("Expected reassign but was {:?}", other),
        };

        assert_eq!(
            *core_cond,
            Core::Id {
                lit: String::from("cond")
            }
        );
        assert_eq!(
            *core_then,
            Core::Id {
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
            Ok(Core::IfElse { cond, then, el }) => (cond, then, el),
            other => panic!("Expected reassign but was {:?}", other),
        };

        assert_eq!(
            *core_cond,
            Core::Id {
                lit: String::from("cond")
            }
        );
        assert_eq!(
            *core_then,
            Core::Id {
                lit: String::from("then")
            }
        );
        assert_eq!(
            *core_else,
            Core::Id {
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
            Ok(Core::While { cond, body }) => (cond, body),
            other => panic!("Expected reassign but was {:?}", other),
        };

        assert_eq!(
            *core_cond,
            Core::Id {
                lit: String::from("cond")
            }
        );
        assert_eq!(
            *core_body,
            Core::ENum {
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

        let (core_expr, core_col, core_body) = match gen(&ASTTy::from(&for_stmt)) {
            Ok(Core::For { expr, col, body }) => (expr, col, body),
            other => panic!("Expected for but was {:?}", other),
        };

        assert_eq!(
            *core_expr,
            Core::Id {
                lit: String::from("expr_1")
            }
        );
        assert_eq!(
            *core_col,
            Core::Id {
                lit: String::from("col")
            }
        );
        assert_eq!(
            *core_body,
            Core::Id {
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
            Ok(Core::FunctionCall { function, args }) => {
                assert_eq!(
                    *function,
                    Core::Id {
                        lit: String::from("range")
                    }
                );
                (args[0].clone(), args[1].clone(), args[2].clone())
            }
            other => panic!("Expected range but was {:?}", other),
        };

        assert_eq!(
            from,
            Core::Id {
                lit: String::from("a")
            }
        );
        assert_eq!(
            to,
            Core::Id {
                lit: String::from("b")
            }
        );
        assert_eq!(
            step,
            Core::Int {
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
            Ok(Core::FunctionCall { function, args }) => {
                assert_eq!(
                    *function,
                    Core::Id {
                        lit: String::from("range")
                    }
                );
                (args[0].clone(), args[1].clone(), args[2].clone())
            }
            other => panic!("Expected range but was {:?}", other),
        };

        assert_eq!(
            from,
            Core::Id {
                lit: String::from("a")
            }
        );
        assert_eq!(
            to,
            Core::Add {
                left: Box::from(Core::Id {
                    lit: String::from("b")
                }),
                right: Box::from(Core::Int {
                    int: String::from("1")
                }),
            }
        );
        assert_eq!(
            step,
            Core::Int {
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
            Ok(Core::FunctionCall { function, args }) => {
                assert_eq!(
                    *function,
                    Core::Id {
                        lit: String::from("range")
                    }
                );
                (args[0].clone(), args[1].clone(), args[2].clone())
            }
            other => panic!("Expected range but was {:?}", other),
        };

        assert_eq!(
            from,
            Core::Id {
                lit: String::from("a")
            }
        );
        assert_eq!(
            to,
            Core::Id {
                lit: String::from("b")
            }
        );
        assert_eq!(
            step,
            Core::Id {
                lit: String::from("c")
            }
        );
    }
}
