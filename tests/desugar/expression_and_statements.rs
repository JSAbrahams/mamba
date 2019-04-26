use mamba::core::construct::Core;
use mamba::desugar::desugar;
use mamba::parser::ast::ASTNode;
use mamba::parser::ast::ASTNodePos;

#[test]
fn break_verify() {
    let _break = to_pos!(ASTNode::Break);
    let core = desugar(&_break);
    assert_eq!(core, Core::Break);
}

#[test]
fn continue_verify() {
    let _continue = to_pos!(ASTNode::Continue);
    let core = desugar(&_continue);
    assert_eq!(core, Core::Continue);
}

#[test]
fn pass_verify() {
    let pass = to_pos!(ASTNode::Pass);
    let core = desugar(&pass);
    assert_eq!(core, Core::Pass);
}

#[test]
fn print_verify() {
    let expr = to_pos!(ASTNode::Str { lit: String::from("a") });
    let print_stmt = to_pos!(ASTNode::Print { expr });

    let expr_core = match desugar(&print_stmt) {
        Core::Print { expr } => expr,
        other => panic!("Expected print but got: {:?}", other)
    };

    assert_eq!(*expr_core, Core::Str { _str: String::from("a") });
}

#[test]
fn return_verify() {
    let expr = to_pos!(ASTNode::Str { lit: String::from("a") });
    let print_stmt = to_pos!(ASTNode::Return { expr });

    let expr_core = match desugar(&print_stmt) {
        Core::Return { expr } => expr,
        other => panic!("Expected print but got: {:?}", other)
    };

    assert_eq!(*expr_core, Core::Str { _str: String::from("a") });
}

#[test]
fn return_empty_verify() {
    let print_stmt = to_pos!(ASTNode::ReturnEmpty);

    let expr_core = match desugar(&print_stmt) {
        Core::Return { expr } => expr,
        other => panic!("Expected print but got: {:?}", other)
    };

    assert_eq!(*expr_core, Core::Empty);
}

#[test]
fn init_verify() {
    let _break = to_pos!(ASTNode::Init);
    let core = desugar(&_break);
    assert_eq!(core, Core::Id { lit: String::from("init") });
}

#[test]
fn self_verify() {
    let _break = to_pos!(ASTNode::_Self);
    let core = desugar(&_break);
    assert_eq!(core, Core::Id { lit: String::from("self") });
}

#[test]
fn import_verify() {
    let _break = to_pos!(ASTNode::Import {
        import: vec![to_pos_unboxed!(ASTNode::Id { lit: String::from("a") })],
        _as:    vec![to_pos_unboxed!(ASTNode::Id { lit: String::from("b") })]
    });
    let core = desugar(&_break);
    assert_eq!(core, Core::Import {
        import: vec![Core::Id { lit: String::from("a") }],
        _as:    vec![Core::Id { lit: String::from("b") }]
    });
}

#[test]
fn from_import_as_verify() {
    let _break = to_pos!(ASTNode::FromImport {
        id:     to_pos!(ASTNode::Id { lit: String::from("f") }),
        import: to_pos!(ASTNode::Import {
            import: vec![to_pos_unboxed!(ASTNode::Id { lit: String::from("a") })],
            _as:    vec![to_pos_unboxed!(ASTNode::Id { lit: String::from("b") })]
        })
    });

    let core = desugar(&_break);
    assert_eq!(core, Core::FromImport {
        from:   Box::from(Core::Id { lit: String::from("f") }),
        import: vec![Core::Id { lit: String::from("a") }],
        _as:    vec![Core::Id { lit: String::from("b") }]
    });
}
