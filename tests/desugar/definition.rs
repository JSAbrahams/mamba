use mamba::core::construct::Core;
use mamba::desugar::desugar;
use mamba::parser::ast::ASTNode;
use mamba::parser::ast::ASTNodePos;
use std::panic;

#[test]
fn reassign_verify() {
    let left = to_pos!(ASTNode::Id { lit: String::from("something") });
    let right = to_pos!(ASTNode::Id { lit: String::from("other") });
    let reassign = to_pos!(ASTNode::Reassign { left, right });

    let (left, right) = match desugar(&reassign) {
        Ok(Core::Assign { left, right }) => (left, right),
        other => panic!("Expected reassign but was {:?}", other)
    };

    assert_eq!(*left, Core::Id { lit: String::from("something") });
    assert_eq!(*right, Core::Id { lit: String::from("other") });
}

#[test]
fn variable_private_def_verify() {
    let definition = to_pos!(ASTNode::VariableDef {
        ofmut:         false,
        id_maybe_type: to_pos!(ASTNode::Id { lit: String::from("d") }),
        expression:    Some(to_pos!(ASTNode::Int { lit: String::from("98") })),
        forward:       vec![]
    });
    let def = to_pos!(ASTNode::Def { private: false, definition });

    let (private, id, right) = match desugar(&def) {
        Ok(Core::VarDef { private, id, right }) => (private, id, right),
        other => panic!("Expected var def but got: {:?}.", other)
    };

    assert_eq!(private, false);
    assert_eq!(*id, Core::Id { lit: String::from("d") });
    assert_eq!(*right, Core::Int { int: String::from("98") });
}

#[test]
fn variable_def_verify() {
    let definition = to_pos!(ASTNode::VariableDef {
        ofmut:         false,
        id_maybe_type: to_pos!(ASTNode::Id { lit: String::from("d") }),
        expression:    Some(to_pos!(ASTNode::Int { lit: String::from("98") })),
        forward:       vec![]
    });
    let def = to_pos!(ASTNode::Def { private: true, definition });

    let (private, id, right) = match desugar(&def) {
        Ok(Core::VarDef { private, id, right }) => (private, id, right),
        other => panic!("Expected var def but got: {:?}.", other)
    };

    assert_eq!(private, true);
    assert_eq!(*id, Core::Id { lit: String::from("d") });
    assert_eq!(*right, Core::Int { int: String::from("98") });
}

#[test]
fn tuple_def_verify() {
    let elements = vec![
        to_pos_unboxed!(ASTNode::Id { lit: String::from("a") }),
        to_pos_unboxed!(ASTNode::Id { lit: String::from("b") }),
    ];
    let expressions = vec![
        to_pos_unboxed!(ASTNode::Id { lit: String::from("c") }),
        to_pos_unboxed!(ASTNode::Id { lit: String::from("d") }),
    ];
    let definition = to_pos!(ASTNode::VariableDef {
        ofmut:         false,
        id_maybe_type: to_pos!(ASTNode::Tuple { elements }),
        expression:    Some(to_pos!(ASTNode::Tuple { elements: expressions })),
        forward:       vec![]
    });
    let def = to_pos!(ASTNode::Def { private: true, definition });

    let (private, id, right) = match desugar(&def) {
        Ok(Core::VarDef { private, id, right }) => (private, id, right),
        other => panic!("Expected var def but got: {:?}.", other)
    };

    assert_eq!(private, true);
    let elements = vec![Core::Id { lit: String::from("a") }, Core::Id { lit: String::from("b") }];
    assert_eq!(*id, Core::Tuple { elements });
    let expressions =
        vec![Core::Id { lit: String::from("c") }, Core::Id { lit: String::from("d") }];
    assert_eq!(*right, Core::Tuple { elements: expressions });
}

#[test]
fn variable_def_none_verify() {
    let definition = to_pos!(ASTNode::VariableDef {
        ofmut:         false,
        id_maybe_type: to_pos!(ASTNode::Id { lit: String::from("d") }),
        expression:    None,
        forward:       vec![]
    });
    let def = to_pos!(ASTNode::Def { private: true, definition });

    let (private, id, right) = match desugar(&def) {
        Ok(Core::VarDef { private, id, right }) => (private, id, right),
        other => panic!("Expected var def but got: {:?}.", other)
    };

    assert_eq!(private, true);
    assert_eq!(*id, Core::Id { lit: String::from("d") });
    assert_eq!(*right, Core::None);
}

#[test]
fn tuple_def_none_verify() {
    let elements = vec![
        to_pos_unboxed!(ASTNode::Id { lit: String::from("a") }),
        to_pos_unboxed!(ASTNode::Id { lit: String::from("b") }),
    ];
    let definition = to_pos!(ASTNode::VariableDef {
        ofmut:         false,
        id_maybe_type: to_pos!(ASTNode::Tuple { elements }),
        expression:    None,
        forward:       vec![]
    });

    let def = to_pos!(ASTNode::Def { private: true, definition });
    let (private, id, right) = match desugar(&def) {
        Ok(Core::VarDef { private, id, right }) => (private, id, right),
        other => panic!("Expected var def but got: {:?}.", other)
    };

    assert_eq!(private, true);
    let elements = vec![Core::Id { lit: String::from("a") }, Core::Id { lit: String::from("b") }];
    assert_eq!(*id, Core::Tuple { elements });
    assert_eq!(*right, Core::Tuple { elements: vec![Core::None, Core::None] });
}

#[test]
fn fun_def_verify() {
    let definition = to_pos!(ASTNode::FunDef {
        id:       to_pos!(ASTNode::Id { lit: String::from("fun") }),
        pure:     false,
        fun_args: vec![
            to_pos_unboxed!(ASTNode::FunArg {
                vararg:        false,
                id_maybe_type: to_pos!(ASTNode::Id { lit: String::from("arg1") }),
                default:       None
            }),
            to_pos_unboxed!(ASTNode::FunArg {
                vararg:        true,
                id_maybe_type: to_pos!(ASTNode::Id { lit: String::from("arg2") }),
                default:       None
            })
        ],
        ret_ty:   None,
        raises:   vec![],
        body:     None
    });
    let def = to_pos!(ASTNode::Def { private: false, definition });

    let (private, id, args, body) = match desugar(&def) {
        Ok(Core::FunDef { private, id, args, body }) => (private, id, args, body),
        other => panic!("Expected fun def but got: {:?}.", other)
    };

    assert_eq!(private, false);
    assert_eq!(*id, Core::Id { lit: String::from("fun") });

    assert_eq!(args.len(), 2);
    assert_eq!(args[0], Core::FunArg {
        vararg:  false,
        id:      Box::from(Core::Id { lit: String::from("arg1") }),
        default: Box::from(Core::Empty)
    });
    assert_eq!(args[1], Core::FunArg {
        vararg:  true,
        id:      Box::from(Core::Id { lit: String::from("arg2") }),
        default: Box::from(Core::Empty)
    });
    assert_eq!(*body, Core::Empty);
}

#[test]
fn fun_def_default_arg_verify() {
    let definition = to_pos!(ASTNode::FunDef {
        id:       to_pos!(ASTNode::Id { lit: String::from("fun") }),
        pure:     false,
        fun_args: vec![to_pos_unboxed!(ASTNode::FunArg {
            vararg:        false,
            id_maybe_type: to_pos!(ASTNode::Id { lit: String::from("arg1") }),
            default:       Some(to_pos!(ASTNode::Str { lit: String::from("asdf") }))
        })],
        ret_ty:   None,
        raises:   vec![],
        body:     None
    });
    let def = to_pos!(ASTNode::Def { private: false, definition });

    let (private, id, args, body) = match desugar(&def) {
        Ok(Core::FunDef { private, id, args, body }) => (private, id, args, body),
        other => panic!("Expected fun def but got: {:?}.", other)
    };

    assert_eq!(private, false);
    assert_eq!(*id, Core::Id { lit: String::from("fun") });

    assert_eq!(args.len(), 1);
    assert_eq!(args[0], Core::FunArg {
        vararg:  false,
        id:      Box::from(Core::Id { lit: String::from("arg1") }),
        default: Box::from(Core::Str { _str: String::from("asdf") })
    });
    assert_eq!(*body, Core::Empty);
}

#[test]
fn fun_def_with_body_verify() {
    let definition = to_pos!(ASTNode::FunDef {
        id:       to_pos!(ASTNode::Id { lit: String::from("fun") }),
        pure:     false,
        fun_args: vec![
            to_pos_unboxed!(ASTNode::Id { lit: String::from("arg1") }),
            to_pos_unboxed!(ASTNode::Id { lit: String::from("arg2") })
        ],
        ret_ty:   None,
        raises:   vec![],
        body:     Some(to_pos!(ASTNode::Real { lit: String::from("2.4") }))
    });
    let def = to_pos!(ASTNode::Def { private: false, definition });

    let (private, id, args, body) = match desugar(&def) {
        Ok(Core::FunDef { private, id, args, body }) => (private, id, args, body),
        other => panic!("Expected fun def but got: {:?}.", other)
    };

    assert_eq!(private, false);
    assert_eq!(*id, Core::Id { lit: String::from("fun") });

    assert_eq!(args.len(), 2);
    assert_eq!(args[0], Core::Id { lit: String::from("arg1") });
    assert_eq!(args[1], Core::Id { lit: String::from("arg2") });
    assert_eq!(*body, Core::Float { float: String::from("2.4") });
}

#[test]
fn anon_fun_verify() {
    let anon_fun = to_pos!(ASTNode::AnonFun {
        args: vec![
            to_pos_unboxed!(ASTNode::Id { lit: String::from("first") }),
            to_pos_unboxed!(ASTNode::Id { lit: String::from("second") })
        ],
        body: to_pos!(ASTNode::Str { lit: String::from("this_string") })
    });

    let (args, body) = match desugar(&anon_fun) {
        Ok(Core::AnonFun { args, body }) => (args, body),
        other => panic!("Expected anon fun but got: {:?}.", other)
    };

    assert_eq!(args.len(), 2);
    assert_eq!(args[0], Core::Id { lit: String::from("first") });
    assert_eq!(args[1], Core::Id { lit: String::from("second") });
    assert_eq!(*body, Core::Str { _str: String::from("this_string") });
}

#[test]
fn top_level_var_def_panic_verify() {
    let var_def = to_pos!(ASTNode::VariableDef {
        ofmut:         false,
        id_maybe_type: to_pos!(ASTNode::Pass),
        expression:    None,
        forward:       vec![]
    });

    panic::set_hook(Box::new(|_info| {}));
    let result = std::panic::catch_unwind(|| desugar(&var_def));
    assert!(result.is_err());
}

#[test]
fn top_level_fun_def_panic_verify() {
    let var_def = to_pos!(ASTNode::FunDef {
        id:       Box::from(to_pos!(ASTNode::Pass)),
        pure:     false,
        fun_args: vec![],
        ret_ty:   None,
        raises:   vec![],
        body:     None
    });

    panic::set_hook(Box::new(|_info| {}));
    let result = std::panic::catch_unwind(|| desugar(&var_def));
    assert!(result.is_err());
}
