use mamba::core::construct::Core;
use mamba::desugar::desugar;
use mamba::parser::ast::ASTNode;
use mamba::parser::ast::ASTNodePos;

#[test]
fn reassign_verify() {
    let left = to_pos!(ASTNode::Id { lit: String::from("something") });
    let right = to_pos!(ASTNode::Id { lit: String::from("other") });
    let reassign = to_pos!(ASTNode::Reassign { left, right });

    let (left, right) = match desugar(&reassign) {
        Core::Assign { left, right } => (left, right),
        other => panic!("Expected reassign but was {:?}", other)
    };

    assert_eq!(*left, Core::Id { lit: String::from("something") });
    assert_eq!(*right, Core::Id { lit: String::from("other") });
}

#[test]
fn variable_private_def_verify() {
    let definition = to_pos!(ASTNode::VariableDef {
        mutable:       false,
        ofmut:         false,
        id_maybe_type: to_pos!(ASTNode::Id { lit: String::from("d") }),
        expression:    Some(to_pos!(ASTNode::Int { lit: String::from("98") })),
        forward:       None
    });
    let def = to_pos!(ASTNode::Def { private: false, definition });

    let (private, id, right) = match desugar(&def) {
        Core::VarDef { private, id, right } => (private, id, right),
        other => panic!("Expected var def but got: {:?}.", other)
    };

    assert_eq!(private, false);
    assert_eq!(*id, Core::Id { lit: String::from("d") });
    assert_eq!(*right, Core::Int { int: String::from("98") });
}

#[test]
fn variable_def_verify() {
    let definition = to_pos!(ASTNode::VariableDef {
        mutable:       false,
        ofmut:         false,
        id_maybe_type: to_pos!(ASTNode::Id { lit: String::from("d") }),
        expression:    Some(to_pos!(ASTNode::Int { lit: String::from("98") })),
        forward:       None
    });
    let def = to_pos!(ASTNode::Def { private: true, definition });

    let (private, id, right) = match desugar(&def) {
        Core::VarDef { private, id, right } => (private, id, right),
        other => panic!("Expected var def but got: {:?}.", other)
    };

    assert_eq!(private, true);
    assert_eq!(*id, Core::Id { lit: String::from("d") });
    assert_eq!(*right, Core::Int { int: String::from("98") });
}

#[test]
fn variable_def_empty_verify() {
    let definition = to_pos!(ASTNode::VariableDef {
        mutable:       false,
        ofmut:         false,
        id_maybe_type: to_pos!(ASTNode::Id { lit: String::from("d") }),
        expression:    None,
        forward:       None
    });
    let def = to_pos!(ASTNode::Def { private: true, definition });

    let (private, id, right) = match desugar(&def) {
        Core::VarDef { private, id, right } => (private, id, right),
        other => panic!("Expected var def but got: {:?}.", other)
    };

    assert_eq!(private, true);
    assert_eq!(*id, Core::Id { lit: String::from("d") });
    assert_eq!(*right, Core::Empty);
}

// TODO add tests for default arguments once implemented
// TODO add tests for default values once implemented
#[test]
fn fun_def_verify() {
    let definition = to_pos!(ASTNode::FunDef {
        id:       to_pos!(ASTNode::Id { lit: String::from("fun") }),
        fun_args: vec![
            to_pos_unboxed!(ASTNode::Id { lit: String::from("arg1") }),
            to_pos_unboxed!(ASTNode::Id { lit: String::from("arg2") })
        ],
        ret_ty:   None,
        raises:   None,
        body:     None
    });
    let def = to_pos!(ASTNode::Def { private: false, definition });

    let (private, id, args, body) = match desugar(&def) {
        Core::FunDef { private, id, args, body } => (private, id, args, body),
        other => panic!("Expected fun def but got: {:?}.", other)
    };

    assert_eq!(private, false);
    assert_eq!(*id, Core::Id { lit: String::from("fun") });

    assert_eq!(args.len(), 2);
    assert_eq!(args[0], Core::Id { lit: String::from("arg1") });
    assert_eq!(args[1], Core::Id { lit: String::from("arg2") });
    assert_eq!(*body, Core::Empty);
}

#[test]
fn fun_def_with_body_verify() {
    let definition = to_pos!(ASTNode::FunDef {
        id:       to_pos!(ASTNode::Id { lit: String::from("fun") }),
        fun_args: vec![
            to_pos_unboxed!(ASTNode::Id { lit: String::from("arg1") }),
            to_pos_unboxed!(ASTNode::Id { lit: String::from("arg2") })
        ],
        ret_ty:   None,
        raises:   None,
        body:     Some(to_pos!(ASTNode::Real { lit: String::from("2.4") }))
    });
    let def = to_pos!(ASTNode::Def { private: false, definition });

    let (private, id, args, body) = match desugar(&def) {
        Core::FunDef { private, id, args, body } => (private, id, args, body),
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
        Core::AnonFun { args, body } => (args, body),
        other => panic!("Expected anon fun but got: {:?}.", other)
    };

    assert_eq!(args.len(), 2);
    assert_eq!(args[0], Core::Id { lit: String::from("first") });
    assert_eq!(args[1], Core::Id { lit: String::from("second") });
    assert_eq!(*body, Core::Str { _str: String::from("this_string") });
}
