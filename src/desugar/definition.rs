use crate::core::construct::Core;
use crate::desugar::common::desugar_vec;
use crate::desugar::node::desugar_node;
use crate::desugar::result::DesugarResult;
use crate::desugar::state::Imports;
use crate::desugar::state::State;
use crate::parse::ast::AST;
use crate::parse::ast::Node;

pub fn desugar_definition(ast: &AST, imp: &mut Imports, state: &State) -> DesugarResult {
    // TODO augment function definition in type checker so that it has return type
    // when applicable
    Ok(match &ast.node {
        Node::VariableDef { var, expr: expression, ty, .. } => {
            let var = desugar_node(var, imp, state)?;
            let state = state.in_tup(match var.clone() {
                Core::Tuple { elements } => elements.len(),
                _ => 1
            });

            Core::VarDef {
                var: Box::from(var.clone()),
                ty: match ty {
                    Some(ty) => Some(Box::from(desugar_node(ty, imp, &state)?)),
                    None => None
                },
                expr: match (var, expression) {
                    (_, Some(expr)) => Some(Box::from(desugar_node(expr, imp, &state)?)),
                    (Core::Tuple { elements }, None) =>
                        Some(Box::from(Core::Tuple { elements: vec![Core::None; elements.len()] })),
                    (_, None) => None
                },
            }
        }
        Node::FunDef { id, args: fun_args, body: expression, ret: ret_ty, .. } => Core::FunDef {
            id: Box::from(desugar_node(id, imp, state)?),
            arg: desugar_vec(fun_args, imp, state)?,
            ty: match ret_ty {
                Some(ret_ty) => Some(Box::from(desugar_node(ret_ty, imp, state)?)),
                None => None
            },
            body: if state.interface {
                Box::from(Core::Pass)
            } else {
                // TODO augment AST in type checker
                Box::from(match expression {
                    Some(expr) => desugar_node(expr, imp, &state.expand_ty(true))?,
                    None => Core::Empty
                })
            },
        },
        definition => panic!("Expected definition: {:?}.", definition)
    })
}

#[cfg(test)]
mod test {
    use crate::common::position::Position;
    use crate::core::construct::Core;
    use crate::desugar::desugar;
    use crate::parse::ast::AST;
    use crate::parse::ast::Node;

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
        let reassign = to_pos!(Node::Reassign { left, right });

        let (left, right) = match desugar(&reassign) {
            Ok(Core::Assign { left, right }) => (left, right),
            other => panic!("Expected reassign but was {:?}", other)
        };

        assert_eq!(*left, Core::Id { lit: String::from("something") });
        assert_eq!(*right, Core::Id { lit: String::from("other") });
    }

    #[test]
    fn variable_private_def_verify() {
        let definition = to_pos!(Node::VariableDef {
        mutable:    false,
        var:        to_pos!(Node::Id { lit: String::from("d") }),
        ty:         None,
        expr: Some(to_pos!(Node::Int { lit: String::from("98") })),
        forward:    vec![]
    });

        let (var, ty, expr) = match desugar(&definition) {
            Ok(Core::VarDef { var, ty, expr }) => (var, ty, expr),
            other => panic!("Expected var def but got: {:?}.", other)
        };

        assert_eq!(ty, None);
        assert_eq!(var, Box::from(Core::Id { lit: String::from("d") }));
        assert_eq!(expr, Some(Box::from(Core::Int { int: String::from("98") })));
    }

    #[test]
    fn variable_def_verify() {
        let definition = to_pos!(Node::VariableDef {
        mutable:    false,
        var:        to_pos!(Node::Id { lit: String::from("d") }),
        ty:         None,
        expr: Some(to_pos!(Node::Int { lit: String::from("98") })),
        forward:    vec![]
    });

        let (var, ty, expr) = match desugar(&definition) {
            Ok(Core::VarDef { var, ty, expr }) => (var, ty, expr),
            other => panic!("Expected var def but got: {:?}.", other)
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
        mutable:    false,
        var:        to_pos!(Node::Tuple { elements }),
        ty:         None,
        expr: Some(to_pos!(Node::Tuple { elements: expressions })),
        forward:    vec![]
    });

        let (var, ty, expr) = match desugar(&definition) {
            Ok(Core::VarDef { var, ty, expr }) => (var, ty, expr),
            other => panic!("Expected var def but got: {:?}.", other)
        };

        assert_eq!(ty, None);
        let elements = vec![Core::Id { lit: String::from("a") }, Core::Id { lit: String::from("b") }];
        assert_eq!(var, Box::from(Core::Tuple { elements }));
        let expressions =
            vec![Core::Id { lit: String::from("c") }, Core::Id { lit: String::from("d") }];
        assert_eq!(expr, Some(Box::from(Core::Tuple { elements: expressions })));
    }

    #[test]
    fn variable_def_none_verify() {
        let definition = to_pos!(Node::VariableDef {
        mutable:    false,
        var:        to_pos!(Node::Id { lit: String::from("d") }),
        ty:         None,
        expr: None,
        forward:    vec![]
    });

        let (var, ty, expr) = match desugar(&definition) {
            Ok(Core::VarDef { var, ty, expr }) => (var, ty, expr),
            other => panic!("Expected var def but got: {:?}.", other)
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
        mutable:    false,
        var:        to_pos!(Node::Tuple { elements }),
        ty:         None,
        expr: None,
        forward:    vec![]
    });

        let (var, ty, expr) = match desugar(&definition) {
            Ok(Core::VarDef { var, ty, expr }) => (var, ty, expr),
            other => panic!("Expected var def but got: {:?}.", other)
        };

        assert_eq!(ty, None);
        let elements = vec![Core::Id { lit: String::from("a") }, Core::Id { lit: String::from("b") }];
        assert_eq!(var, Box::from(Core::Tuple { elements }));
        assert_eq!(expr, Some(Box::from(Core::Tuple { elements: vec![Core::None, Core::None] })));
    }

    #[test]
    fn fun_def_verify() {
        let definition = to_pos!(Node::FunDef {
        id:       to_pos!(Node::Id { lit: String::from("fun") }),
        pure:     false,
        args: vec![
            to_pos_unboxed!(Node::FunArg {
                vararg:  false,
                mutable: false,
                var:     to_pos!(Node::Id { lit: String::from("arg1") }),
                ty:      None,
                default: None
            }),
            to_pos_unboxed!(Node::FunArg {
                vararg:  true,
                mutable: false,
                var:     to_pos!(Node::Id { lit: String::from("arg2") }),
                ty:      None,
                default: None
            })
        ],
        ret:   None,
        raises:   vec![],
        body:     None
    });

        let (id, args, body) = match desugar(&definition) {
            Ok(Core::FunDef { id, arg, body, .. }) => (id, arg, body),
            other => panic!("Expected fun def but got: {:?}.", other)
        };

        assert_eq!(*id, Core::Id { lit: String::from("fun") });

        assert_eq!(args.len(), 2);
        assert_eq!(args[0], Core::FunArg {
            vararg: false,
            var: Box::from(Core::Id { lit: String::from("arg1") }),
            ty: None,
            default: None,
        });
        assert_eq!(args[1], Core::FunArg {
            vararg: true,
            var: Box::from(Core::Id { lit: String::from("arg2") }),
            ty: None,
            default: None,
        });
        assert_eq!(*body, Core::Empty);
    }

    #[test]
    fn fun_def_default_arg_verify() {
        let definition = to_pos!(Node::FunDef {
        id:       to_pos!(Node::Id { lit: String::from("fun") }),
        pure:     false,
        args: vec![to_pos_unboxed!(Node::FunArg {
            vararg:  false,
            mutable: false,
            var:     to_pos!(Node::Id { lit: String::from("arg1") }),
            ty:      None,
            default: Some(to_pos!(Node::Str {
                lit:         String::from("asdf"),
                expressions: vec![]
            }))
        })],
        ret:   None,
        raises:   vec![],
        body:     None
    });

        let (id, args, body) = match desugar(&definition) {
            Ok(Core::FunDef { id, arg, body, .. }) => (id, arg, body),
            other => panic!("Expected fun def but got: {:?}.", other)
        };

        assert_eq!(*id, Core::Id { lit: String::from("fun") });

        assert_eq!(args.len(), 1);
        assert_eq!(args[0], Core::FunArg {
            vararg: false,
            var: Box::from(Core::Id { lit: String::from("arg1") }),
            ty: None,
            default: Some(Box::from(Core::Str { string: String::from("asdf") })),
        });
        assert_eq!(*body, Core::Empty);
    }

    #[test]
    fn fun_def_with_body_verify() {
        let definition = to_pos!(Node::FunDef {
        id:       to_pos!(Node::Id { lit: String::from("fun") }),
        pure:     false,
        args: vec![
            to_pos_unboxed!(Node::Id { lit: String::from("arg1") }),
            to_pos_unboxed!(Node::Id { lit: String::from("arg2") })
        ],
        ret:   None,
        raises:   vec![],
        body:     Some(to_pos!(Node::Real { lit: String::from("2.4") }))
    });

        let (id, args, body) = match desugar(&definition) {
            Ok(Core::FunDef { id, arg, body, .. }) => (id, arg, body),
            other => panic!("Expected fun def but got: {:?}.", other)
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

        let (args, body) = match desugar(&anon_fun) {
            Ok(Core::AnonFun { args, body }) => (args, body),
            other => panic!("Expected anon fun but got: {:?}.", other)
        };

        assert_eq!(args.len(), 2);
        assert_eq!(args[0], Core::Id { lit: String::from("first") });
        assert_eq!(args[1], Core::Id { lit: String::from("second") });
        assert_eq!(*body, Core::Str { string: String::from("this_string") });
    }
}
