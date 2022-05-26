use std::ops::Deref;

use crate::ASTTy;
use crate::check::ast::NodeTy;
use crate::check::context::arg;
use crate::check::name::Name;
use crate::generate::ast::node::{Core, CoreFunOp};
use crate::generate::convert::common::convert_vec;
use crate::generate::convert::convert_node;
use crate::generate::convert::state::{Imports, State};
use crate::generate::result::{GenResult, UnimplementedErr};

pub fn convert_def(ast: &ASTTy, imp: &mut Imports, state: &State) -> GenResult {
    match &ast.node {
        NodeTy::VariableDef { var, expr: expression, ty, .. } => {
            let var = convert_node(var, imp, &state.tuple_literal())?;
            let state = state.in_tup(match var.clone() {
                Core::Tuple { elements } => elements.len(),
                _ => 1,
            });

            Ok(if state.def_as_fun_arg {
                Core::FunArg {
                    vararg: false,
                    var: Box::from(var),
                    ty: match ty {
                        Some(ty) => Some(Box::from(convert_node(ty, imp, &state)?)),
                        None => ast.ty.clone().map(|name| name.to_py()).map(Box::from),
                    },
                    default: match expression {
                        Some(expression) => Some(Box::from(convert_node(expression, imp, &state)?)),
                        None => None,
                    },
                }
            } else {
                Core::VarDef {
                    var: Box::from(var.clone()),
                    ty: match ty {
                        Some(ty) if !matches!(var, Core::TupleLiteral { .. }) => {
                            Some(Box::from(convert_node(ty, imp, &state)?))
                        }
                        _ => match expression {
                            Some(expr) => expr.clone().ty.map(|name| name.to_py()).map(Box::from),
                            None => None
                        },
                    },
                    expr: match (var, expression) {
                        (_, Some(expr)) => Some(Box::from(convert_node(expr, imp, &state)?)),
                        (Core::TupleLiteral { elements }, None) => Some(Box::from(Core::Tuple {
                            elements: vec![Core::None; elements.len()],
                        })),
                        (_, None) => None,
                    },
                }
            })
        }
        NodeTy::FunDef { id, args: fun_args, body: expression, ret: ret_ty, .. } => {
            let arg = convert_vec(fun_args, imp, state)?;
            let ty = match ret_ty {
                Some(ret_ty) => Some(Box::from(convert_node(ret_ty, imp, state)?)),
                None => None,
            };
            let body = if state.interface {
                Box::from(Core::Pass)
            } else {
                Box::from(match expression {
                    Some(expr) => convert_node(expr, imp, &state.expand_ty(true))?,
                    None => Core::Pass,
                })
            };

            let c_id = Box::from(convert_node(id, imp, state)?);
            match c_id.deref() {
                Core::Id { lit } => Ok(if let Some(op) = CoreFunOp::from(lit.as_str()) {
                    Core::FunDefOp { op, arg, ty, body }
                } else {
                    Core::FunDef { id: c_id.clone(), arg, ty, body }
                }),
                _ => Err(UnimplementedErr::new(id, "Non-id function"))
            }
        }
        NodeTy::FunArg { vararg, var, ty, default, .. } => Ok(Core::FunArg {
            vararg: *vararg,
            var: Box::from(convert_node(var, imp, state)?),
            ty: match ty {
                Some(ty) if state.expand_ty => match &var.node {
                    NodeTy::Id { lit } if lit == &String::from(arg::SELF) => None,
                    _ => Some(Box::from(convert_node(ty, imp, state)?)),
                },
                _ => None,
            },
            default: match default {
                Some(default) => Some(Box::from(convert_node(default, imp, state)?)),
                None => None,
            },
        }),
        definition => {
            let msg = format!("Expected definition: {:?}", definition);
            Err(UnimplementedErr::new(ast, &msg))
        }
    }
}

impl Name {
    fn to_py(&self) -> Core {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use crate::ASTTy;
    use crate::common::position::Position;
    use crate::generate::ast::node::{Core, CoreOp};
    use crate::generate::gen;
    use crate::parse::ast::AST;
    use crate::parse::ast::Node;
    use crate::parse::ast::node_op::NodeOp;

    macro_rules! to_pos_unboxed {
        ($node:expr) => {{
            AST { pos: Position::default(), node: $node }
        }};
    }

    macro_rules! to_pos {
        ($node:expr) => {{
            Box::from(to_pos_unboxed!($node))
        }};
    }

    #[test]
    fn reassign_verify() {
        let left = to_pos!(Node::Id { lit: String::from("something") });
        let right = to_pos!(Node::Id { lit: String::from("other") });
        let reassign = to_pos!(Node::Reassign { left, right, op: NodeOp::Assign });

        let (left, right, op) = match gen(&ASTTy::from(&reassign)) {
            Ok(Core::Assign { left, right, op }) => (left, right, op),
            other => panic!("Expected reassign but was {:?}", other),
        };

        assert_eq!(*left, Core::Id { lit: String::from("something") });
        assert_eq!(*right, Core::Id { lit: String::from("other") });
        assert_eq!(op, CoreOp::Assign);
    }

    #[test]
    fn variable_private_def_verify() {
        let definition = to_pos!(Node::VariableDef {
            mutable: false,
            var: to_pos!(Node::Id { lit: String::from("d") }),
            ty: None,
            expr: Some(to_pos!(Node::Int { lit: String::from("98") })),
            forward: vec![]
        });

        let (var, ty, expr) = match gen(&ASTTy::from(&definition)) {
            Ok(Core::VarDef { var, ty, expr }) => (var, ty, expr),
            other => panic!("Expected var def but got: {:?}.", other),
        };

        assert_eq!(ty, None);
        assert_eq!(var, Box::from(Core::Id { lit: String::from("d") }));
        assert_eq!(expr, Some(Box::from(Core::Int { int: String::from("98") })));
    }

    #[test]
    fn variable_def_verify() {
        let definition = to_pos!(Node::VariableDef {
            mutable: false,
            var: to_pos!(Node::Id { lit: String::from("d") }),
            ty: None,
            expr: Some(to_pos!(Node::Int { lit: String::from("98") })),
            forward: vec![]
        });

        let (var, ty, expr) = match gen(&ASTTy::from(&definition)) {
            Ok(Core::VarDef { var, ty, expr }) => (var, ty, expr),
            other => panic!("Expected var def but got: {:?}.", other),
        };

        assert_eq!(ty, None);
        assert_eq!(var, Box::from(Core::Id { lit: String::from("d") }));
        assert_eq!(expr, Some(Box::from(Core::Int { int: String::from("98") })));
    }

    #[test]
    fn tuple_def_verify() {
        let elements = vec![
            to_pos_unboxed!(Node::Id { lit: String::from("a") }),
            to_pos_unboxed!(Node::Id { lit: String::from("b") }),
        ];
        let expressions = vec![
            to_pos_unboxed!(Node::Id { lit: String::from("c") }),
            to_pos_unboxed!(Node::Id { lit: String::from("d") }),
        ];
        let definition = to_pos!(Node::VariableDef {
            mutable: false,
            var: to_pos!(Node::Tuple { elements }),
            ty: None,
            expr: Some(to_pos!(Node::Tuple { elements: expressions })),
            forward: vec![]
        });

        let (var, ty, expr) = match gen(&ASTTy::from(&definition)) {
            Ok(Core::VarDef { var, ty, expr }) => (var, ty, expr),
            other => panic!("Expected var def but got: {:?}.", other),
        };

        assert_eq!(ty, None);
        let elements =
            vec![Core::Id { lit: String::from("a") }, Core::Id { lit: String::from("b") }];
        assert_eq!(var, Box::from(Core::TupleLiteral { elements }));
        let expressions =
            vec![Core::Id { lit: String::from("c") }, Core::Id { lit: String::from("d") }];
        assert_eq!(expr, Some(Box::from(Core::Tuple { elements: expressions })));
    }

    #[test]
    fn variable_def_none_verify() {
        let definition = to_pos!(Node::VariableDef {
            mutable: false,
            var: to_pos!(Node::Id { lit: String::from("d") }),
            ty: None,
            expr: None,
            forward: vec![]
        });

        let (var, ty, expr) = match gen(&ASTTy::from(&definition)) {
            Ok(Core::VarDef { var, ty, expr }) => (var, ty, expr),
            other => panic!("Expected var def but got: {:?}.", other),
        };

        assert_eq!(ty, None);
        assert_eq!(var, Box::from(Core::Id { lit: String::from("d") }));
        assert_eq!(expr, None);
    }

    #[test]
    fn tuple_def_none_verify() {
        let elements = vec![
            to_pos_unboxed!(Node::Id { lit: String::from("a") }),
            to_pos_unboxed!(Node::Id { lit: String::from("b") }),
        ];
        let definition = to_pos!(Node::VariableDef {
            mutable: false,
            var: to_pos!(Node::Tuple { elements }),
            ty: None,
            expr: None,
            forward: vec![]
        });

        let (var, ty, expr) = match gen(&ASTTy::from(&definition)) {
            Ok(Core::VarDef { var, ty, expr }) => (var, ty, expr),
            other => panic!("Expected var def but got: {:?}.", other),
        };

        assert_eq!(ty, None);
        let elements =
            vec![Core::Id { lit: String::from("a") }, Core::Id { lit: String::from("b") }];
        assert_eq!(var, Box::from(Core::TupleLiteral { elements }));
        assert_eq!(expr, Some(Box::from(Core::Tuple { elements: vec![Core::None, Core::None] })));
    }

    #[test]
    fn fun_def_verify() {
        let definition = to_pos!(Node::FunDef {
            id: to_pos!(Node::Id { lit: String::from("fun") }),
            pure: false,
            args: vec![
                to_pos_unboxed!(Node::FunArg {
                    vararg: false,
                    mutable: false,
                    var: to_pos!(Node::Id { lit: String::from("arg1") }),
                    ty: None,
                    default: None
                }),
                to_pos_unboxed!(Node::FunArg {
                    vararg: true,
                    mutable: false,
                    var: to_pos!(Node::Id { lit: String::from("arg2") }),
                    ty: None,
                    default: None
                })
            ],
            ret: None,
            raises: vec![],
            body: None
        });

        let (id, args, body) = match gen(&ASTTy::from(&definition)) {
            Ok(Core::FunDef { id, arg, body, .. }) => (id, arg, body),
            other => panic!("Expected fun def but got: {:?}.", other),
        };

        assert_eq!(*id, Core::Id { lit: String::from("fun") });

        assert_eq!(args.len(), 2);
        assert_eq!(
            args[0],
            Core::FunArg {
                vararg: false,
                var: Box::from(Core::Id { lit: String::from("arg1") }),
                ty: None,
                default: None,
            }
        );
        assert_eq!(
            args[1],
            Core::FunArg {
                vararg: true,
                var: Box::from(Core::Id { lit: String::from("arg2") }),
                ty: None,
                default: None,
            }
        );
        assert_eq!(*body, Core::Pass);
    }

    #[test]
    fn fun_def_default_arg_verify() {
        let definition = to_pos!(Node::FunDef {
            id: to_pos!(Node::Id { lit: String::from("fun") }),
            pure: false,
            args: vec![to_pos_unboxed!(Node::FunArg {
                vararg: false,
                mutable: false,
                var: to_pos!(Node::Id { lit: String::from("arg1") }),
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
            Ok(Core::FunDef { id, arg, body, .. }) => (id, arg, body),
            other => panic!("Expected fun def but got: {:?}.", other),
        };

        assert_eq!(*id, Core::Id { lit: String::from("fun") });

        assert_eq!(args.len(), 1);
        assert_eq!(
            args[0],
            Core::FunArg {
                vararg: false,
                var: Box::from(Core::Id { lit: String::from("arg1") }),
                ty: None,
                default: Some(Box::from(Core::Str { string: String::from("asdf") })),
            }
        );
        assert_eq!(*body, Core::Pass);
    }

    #[test]
    fn fun_def_with_body_verify() {
        let definition = to_pos!(Node::FunDef {
            id: to_pos!(Node::Id { lit: String::from("fun") }),
            pure: false,
            args: vec![
                to_pos_unboxed!(Node::Id { lit: String::from("arg1") }),
                to_pos_unboxed!(Node::Id { lit: String::from("arg2") })
            ],
            ret: None,
            raises: vec![],
            body: Some(to_pos!(Node::Real { lit: String::from("2.4") }))
        });

        let (id, args, body) = match gen(&ASTTy::from(&definition)) {
            Ok(Core::FunDef { id, arg, body, .. }) => (id, arg, body),
            other => panic!("Expected fun def but got: {:?}.", other),
        };

        assert_eq!(*id, Core::Id { lit: String::from("fun") });

        assert_eq!(args.len(), 2);
        assert_eq!(args[0], Core::Id { lit: String::from("arg1") });
        assert_eq!(args[1], Core::Id { lit: String::from("arg2") });
        assert_eq!(*body, Core::Float { float: String::from("2.4") });
    }

    #[test]
    fn anon_fun_verify() {
        let anon_fun = to_pos!(Node::AnonFun {
            args: vec![
                to_pos_unboxed!(Node::Id { lit: String::from("first") }),
                to_pos_unboxed!(Node::Id { lit: String::from("second") })
            ],
            body: to_pos!(Node::Str { lit: String::from("this_string"), expressions: vec![] })
        });

        let (args, body) = match gen(&ASTTy::from(&anon_fun)) {
            Ok(Core::AnonFun { args, body }) => (args, body),
            other => panic!("Expected anon fun but got: {:?}.", other),
        };

        assert_eq!(args.len(), 2);
        assert_eq!(args[0], Core::Id { lit: String::from("first") });
        assert_eq!(args[1], Core::Id { lit: String::from("second") });
        assert_eq!(*body, Core::Str { string: String::from("this_string") });
    }
}
