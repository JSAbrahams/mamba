use std::ops::Deref;

use crate::backend::python::ast::node::{CoreFunOp, PythonCore};
use crate::backend::python::convert::common::convert_vec;
use crate::backend::python::convert::convert_node;
use crate::backend::python::convert::state::{Imports, State};
use crate::backend::python::name::ToPy;
use crate::backend::python::result::{GenResult, UnimplementedErr};
use crate::check::ast::NodeTy;
use crate::check::context::arg::python::SELF;
use crate::check::context::function;
use crate::{ASTTy, Context};

pub fn convert_def(ast: &ASTTy, imp: &mut Imports, state: &State, ctx: &Context) -> GenResult {
    match &ast.node {
        NodeTy::VariableDef { var, expr, ty, .. } => {
            let var = convert_node(var, imp, &state.tuple_literal(), ctx)?;
            let state = state.in_tup(match var.clone() {
                PythonCore::Tuple { elements } => elements.len(),
                _ => 1,
            });

            let annotate = state.annotate
                && state.expand_ty
                && !matches!(var, PythonCore::TupleLiteral { .. });
            let ty = match (ty, expr) {
                (Some(ty), _) if annotate => Some(Box::from(ty.to_py(imp))),
                (_, Some(expr)) if annotate => {
                    expr.clone().ty.map(|name| name.to_py(imp)).map(Box::from)
                }
                _ => None,
            };

            Ok(if state.def_as_fun_arg {
                let default = match expr {
                    Some(expression) => {
                        Some(Box::from(convert_node(expression, imp, &state, ctx)?))
                    }
                    None => None,
                };
                PythonCore::FunArg {
                    vararg: false,
                    var: Box::from(var),
                    ty,
                    default,
                }
            } else {
                let expr = match (&var, expr) {
                    (_, Some(expr)) => match convert_node(expr, imp, &state, ctx)? {
                        PythonCore::IfElse { .. } | PythonCore::Match { .. } => {
                            // redo convert but with assign to state
                            let state = state.must_assign_to(Some(&var.clone()), expr.ty.clone());
                            return convert_node(expr, imp, &state, ctx);
                        }
                        other => Some(Box::from(other)),
                    },
                    (PythonCore::TupleLiteral { elements }, None) => {
                        Some(Box::from(PythonCore::Tuple {
                            elements: vec![PythonCore::None; elements.len()],
                        }))
                    }
                    (_, None) => None,
                };
                PythonCore::VarDef {
                    var: Box::from(var),
                    ty,
                    expr,
                }
            })
        }
        NodeTy::FunDef {
            id,
            args: fun_args,
            body: expression,
            ret: ret_ty,
            ..
        } => {
            let arg = convert_vec(fun_args, imp, state, ctx)?;
            let ty = match ret_ty {
                Some(ret_ty) if state.annotate => Some(Box::from(ret_ty.to_py(imp))),
                _ => None,
            };
            let (dec, body) = if state.interface && expression.is_none() {
                imp.add_from_import("abc", "abstractmethod");
                (
                    vec![String::from("abstractmethod")],
                    Box::from(PythonCore::Pass),
                )
            } else {
                (
                    vec![],
                    Box::from(match expression {
                        Some(expr) => convert_node(
                            expr,
                            imp,
                            &state.expand_ty(true).is_last_must_be_ret(ty.is_some()),
                            ctx,
                        )?,
                        None => PythonCore::Pass,
                    }),
                )
            };

            let c_id = Box::from(convert_node(id, imp, state, ctx)?);
            match c_id.deref() {
                PythonCore::Id { lit } => Ok(if let Some(op) = CoreFunOp::from(lit.as_str()) {
                    PythonCore::FunDefOp { op, arg, ty, body }
                } else {
                    let id = match c_id.as_ref() {
                        PythonCore::Id { ref lit, .. } => match lit.as_str() {
                            "size" => String::from("__size__"),
                            function::python::INIT => String::from("__init__"),
                            other => String::from(other),
                        },
                        other => {
                            let msg = format!("Expected function identifier, was {other:?}");
                            return Err(Box::from(UnimplementedErr::new(id, &msg)));
                        }
                    };

                    PythonCore::FunDef {
                        dec,
                        id,
                        arg,
                        ty,
                        body,
                    }
                }),
                _ => Err(Box::from(UnimplementedErr::new(id, "Non-id function"))),
            }
        }
        NodeTy::FunArg {
            vararg,
            var,
            ty,
            default,
            ..
        } => {
            let var = convert_node(var, imp, state, ctx)?;
            let annotate = state.annotate
                && state.expand_ty
                && var
                    != PythonCore::Id {
                        lit: String::from(SELF),
                    };

            Ok(PythonCore::FunArg {
                vararg: *vararg,
                var: Box::from(var),
                ty: if annotate {
                    ty.as_ref().map(|ty| ty.to_py(imp)).map(Box::from)
                } else {
                    None
                },
                default: match default {
                    Some(default) => Some(Box::from(convert_node(default, imp, state, ctx)?)),
                    None => None,
                },
            })
        }
        definition => {
            let msg = format!("Expected definition, was {definition:?}");
            Err(Box::from(UnimplementedErr::new(ast, &msg)))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::backend::python::ast::node::{CoreOp, PythonCore};
    use crate::backend::python::gen;
    use crate::common::position::Position;
    use crate::parse::ast::node_op::NodeOp;
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
    fn reassign_verify() {
        let left = to_pos!(Node::Id {
            lit: String::from("something")
        });
        let right = to_pos!(Node::Id {
            lit: String::from("other")
        });
        let reassign = to_pos!(Node::Reassign {
            left,
            right,
            op: NodeOp::Assign
        });

        let (left, right, op) = match gen(&ASTTy::from(&reassign)) {
            Ok(PythonCore::Assign { left, right, op }) => (left, right, op),
            other => panic!("Expected reassign but was {other:?}"),
        };

        assert_eq!(
            *left,
            PythonCore::Id {
                lit: String::from("something")
            }
        );
        assert_eq!(
            *right,
            PythonCore::Id {
                lit: String::from("other")
            }
        );
        assert_eq!(op, CoreOp::Assign);
    }

    #[test]
    fn variable_private_def_verify() {
        let definition = to_pos!(Node::VariableDef {
            mutable: false,
            var: to_pos!(Node::Id {
                lit: String::from("d")
            }),
            ty: None,
            expr: Some(to_pos!(Node::Int {
                lit: String::from("98")
            })),
            forward: vec![]
        });

        let (var, ty, expr) = match gen(&ASTTy::from(&definition)) {
            Ok(PythonCore::VarDef { var, ty, expr }) => (var, ty, expr),
            other => panic!("Expected var def but got: {other:?}."),
        };

        assert_eq!(ty, None);
        assert_eq!(
            var,
            Box::from(PythonCore::Id {
                lit: String::from("d")
            })
        );
        assert_eq!(
            expr,
            Some(Box::from(PythonCore::Int {
                int: String::from("98")
            }))
        );
    }

    #[test]
    fn variable_def_verify() {
        let definition = to_pos!(Node::VariableDef {
            mutable: false,
            var: to_pos!(Node::Id {
                lit: String::from("d")
            }),
            ty: None,
            expr: Some(to_pos!(Node::Int {
                lit: String::from("98")
            })),
            forward: vec![]
        });

        let (var, ty, expr) = match gen(&ASTTy::from(&definition)) {
            Ok(PythonCore::VarDef { var, ty, expr }) => (var, ty, expr),
            other => panic!("Expected var def but got: {other:?}."),
        };

        assert_eq!(ty, None);
        assert_eq!(
            var,
            Box::from(PythonCore::Id {
                lit: String::from("d")
            })
        );
        assert_eq!(
            expr,
            Some(Box::from(PythonCore::Int {
                int: String::from("98")
            }))
        );
    }

    #[test]
    fn tuple_def_verify() {
        let elements = vec![
            to_pos_unboxed!(Node::Id {
                lit: String::from("a")
            }),
            to_pos_unboxed!(Node::Id {
                lit: String::from("b")
            }),
        ];
        let expressions = vec![
            to_pos_unboxed!(Node::Id {
                lit: String::from("c")
            }),
            to_pos_unboxed!(Node::Id {
                lit: String::from("d")
            }),
        ];
        let definition = to_pos!(Node::VariableDef {
            mutable: false,
            var: to_pos!(Node::Tuple { elements }),
            ty: None,
            expr: Some(to_pos!(Node::Tuple {
                elements: expressions
            })),
            forward: vec![]
        });

        let (var, ty, expr) = match gen(&ASTTy::from(&definition)) {
            Ok(PythonCore::VarDef { var, ty, expr }) => (var, ty, expr),
            other => panic!("Expected var def but got: {other:?}."),
        };

        assert_eq!(ty, None);
        let elements = vec![
            PythonCore::Id {
                lit: String::from("a"),
            },
            PythonCore::Id {
                lit: String::from("b"),
            },
        ];
        assert_eq!(var, Box::from(PythonCore::TupleLiteral { elements }));
        let expressions = vec![
            PythonCore::Id {
                lit: String::from("c"),
            },
            PythonCore::Id {
                lit: String::from("d"),
            },
        ];
        assert_eq!(
            expr,
            Some(Box::from(PythonCore::Tuple {
                elements: expressions
            }))
        );
    }

    #[test]
    fn variable_def_none_verify() {
        let definition = to_pos!(Node::VariableDef {
            mutable: false,
            var: to_pos!(Node::Id {
                lit: String::from("d")
            }),
            ty: None,
            expr: None,
            forward: vec![]
        });

        let (var, ty, expr) = match gen(&ASTTy::from(&definition)) {
            Ok(PythonCore::VarDef { var, ty, expr }) => (var, ty, expr),
            other => panic!("Expected var def but got: {other:?}."),
        };

        assert_eq!(ty, None);
        assert_eq!(
            var,
            Box::from(PythonCore::Id {
                lit: String::from("d")
            })
        );
        assert_eq!(expr, None);
    }

    #[test]
    fn tuple_def_none_verify() {
        let elements = vec![
            to_pos_unboxed!(Node::Id {
                lit: String::from("a")
            }),
            to_pos_unboxed!(Node::Id {
                lit: String::from("b")
            }),
        ];
        let definition = to_pos!(Node::VariableDef {
            mutable: false,
            var: to_pos!(Node::Tuple { elements }),
            ty: None,
            expr: None,
            forward: vec![]
        });

        let (var, ty, expr) = match gen(&ASTTy::from(&definition)) {
            Ok(PythonCore::VarDef { var, ty, expr }) => (var, ty, expr),
            other => panic!("Expected var def but got: {other:?}."),
        };

        assert_eq!(ty, None);
        let elements = vec![
            PythonCore::Id {
                lit: String::from("a"),
            },
            PythonCore::Id {
                lit: String::from("b"),
            },
        ];
        assert_eq!(var, Box::from(PythonCore::TupleLiteral { elements }));
        assert_eq!(
            expr,
            Some(Box::from(PythonCore::Tuple {
                elements: vec![PythonCore::None, PythonCore::None]
            }))
        );
    }

    #[test]
    fn fun_def_verify() {
        let definition = to_pos!(Node::FunDef {
            id: to_pos!(Node::Id {
                lit: String::from("fun")
            }),
            pure: false,
            args: vec![
                to_pos_unboxed!(Node::FunArg {
                    vararg: false,
                    mutable: false,
                    var: to_pos!(Node::Id {
                        lit: String::from("arg1")
                    }),
                    ty: None,
                    default: None
                }),
                to_pos_unboxed!(Node::FunArg {
                    vararg: true,
                    mutable: false,
                    var: to_pos!(Node::Id {
                        lit: String::from("arg2")
                    }),
                    ty: None,
                    default: None
                })
            ],
            ret: None,
            raises: vec![],
            body: None
        });

        let (id, args, body) = match gen(&ASTTy::from(&definition)) {
            Ok(PythonCore::FunDef { id, arg, body, .. }) => (id, arg, body),
            other => panic!("Expected fun def but got: {other:?}."),
        };

        assert_eq!(*id, String::from("fun"));

        assert_eq!(args.len(), 2);
        assert_eq!(
            args[0],
            PythonCore::FunArg {
                vararg: false,
                var: Box::from(PythonCore::Id {
                    lit: String::from("arg1")
                }),
                ty: None,
                default: None,
            }
        );
        assert_eq!(
            args[1],
            PythonCore::FunArg {
                vararg: true,
                var: Box::from(PythonCore::Id {
                    lit: String::from("arg2")
                }),
                ty: None,
                default: None,
            }
        );
        assert_eq!(*body, PythonCore::Pass);
    }

    #[test]
    fn fun_def_default_arg_verify() {
        let definition = to_pos!(Node::FunDef {
            id: to_pos!(Node::Id {
                lit: String::from("fun")
            }),
            pure: false,
            args: vec![to_pos_unboxed!(Node::FunArg {
                vararg: false,
                mutable: false,
                var: to_pos!(Node::Id {
                    lit: String::from("arg1")
                }),
                ty: None,
                default: Some(to_pos!(Node::Str {
                    lit: String::from("asdf"),
                    expressions: vec![]
                }))
            })],
            ret: None,
            raises: vec![],
            body: None
        });

        let (id, args, body) = match gen(&ASTTy::from(&definition)) {
            Ok(PythonCore::FunDef { id, arg, body, .. }) => (id, arg, body),
            other => panic!("Expected fun def but got: {other:?}."),
        };

        assert_eq!(*id, String::from("fun"));

        assert_eq!(args.len(), 1);
        assert_eq!(
            args[0],
            PythonCore::FunArg {
                vararg: false,
                var: Box::from(PythonCore::Id {
                    lit: String::from("arg1")
                }),
                ty: None,
                default: Some(Box::from(PythonCore::Str {
                    string: String::from("asdf")
                })),
            }
        );
        assert_eq!(*body, PythonCore::Pass);
    }

    #[test]
    fn fun_def_with_body_verify() {
        let definition = to_pos!(Node::FunDef {
            id: to_pos!(Node::Id {
                lit: String::from("fun")
            }),
            pure: false,
            args: vec![
                to_pos_unboxed!(Node::Id {
                    lit: String::from("arg1")
                }),
                to_pos_unboxed!(Node::Id {
                    lit: String::from("arg2")
                })
            ],
            ret: None,
            raises: vec![],
            body: Some(to_pos!(Node::Real {
                lit: String::from("2.4")
            }))
        });

        let (id, args, body) = match gen(&ASTTy::from(&definition)) {
            Ok(PythonCore::FunDef { id, arg, body, .. }) => (id, arg, body),
            other => panic!("Expected fun def but got: {other:?}."),
        };

        assert_eq!(*id, String::from("fun"));

        assert_eq!(args.len(), 2);
        assert_eq!(
            args[0],
            PythonCore::Id {
                lit: String::from("arg1")
            }
        );
        assert_eq!(
            args[1],
            PythonCore::Id {
                lit: String::from("arg2")
            }
        );
        assert_eq!(
            *body,
            PythonCore::Float {
                float: String::from("2.4")
            }
        );
    }

    #[test]
    fn anon_fun_verify() {
        let anon_fun = to_pos!(Node::AnonFun {
            args: vec![
                to_pos_unboxed!(Node::Id {
                    lit: String::from("first")
                }),
                to_pos_unboxed!(Node::Id {
                    lit: String::from("second")
                })
            ],
            body: to_pos!(Node::Str {
                lit: String::from("this_string"),
                expressions: vec![]
            })
        });

        let (args, body) = match gen(&ASTTy::from(&anon_fun)) {
            Ok(PythonCore::AnonFun { args, body }) => (args, body),
            other => panic!("Expected anon fun but got: {other:?}."),
        };

        assert_eq!(args.len(), 2);
        assert_eq!(
            args[0],
            PythonCore::Id {
                lit: String::from("first")
            }
        );
        assert_eq!(
            args[1],
            PythonCore::Id {
                lit: String::from("second")
            }
        );
        assert_eq!(
            *body,
            PythonCore::Str {
                string: String::from("this_string")
            }
        );
    }
}
