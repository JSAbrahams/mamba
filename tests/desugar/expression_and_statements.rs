use mamba::core::construct::Core;
use mamba::desugar::desugar;
use mamba::parser::ast::ASTNode;
use mamba::parser::ast::ASTNodePos;
use std::panic;

#[test]
fn break_verify() {
    let _break = to_pos!(ASTNode::Break);
    assert_eq!(desugar(&_break), Core::Break);
}

#[test]
fn continue_verify() {
    let _continue = to_pos!(ASTNode::Continue);
    assert_eq!(desugar(&_continue), Core::Continue);
}

#[test]
fn pass_verify() {
    let pass = to_pos!(ASTNode::Pass);
    assert_eq!(desugar(&pass), Core::Pass);
}

#[test]
fn print_verify() {
    let expr = to_pos!(ASTNode::Str { lit: String::from("a") });
    let print_stmt = to_pos!(ASTNode::Print { expr });
    assert_eq!(desugar(&print_stmt), Core::Print {
        expr: Box::from(Core::Str { _str: String::from("a") })
    });
}

#[test]
fn return_verify() {
    let expr = to_pos!(ASTNode::Str { lit: String::from("a") });
    let print_stmt = to_pos!(ASTNode::Return { expr });

    assert_eq!(desugar(&print_stmt), Core::Return {
        expr: Box::from(Core::Str { _str: String::from("a") })
    });
}

#[test]
fn return_empty_verify() {
    let print_stmt = to_pos!(ASTNode::ReturnEmpty);
    assert_eq!(desugar(&print_stmt), Core::Return { expr: Box::from(Core::None) });
}

#[test]
fn init_verify() {
    let _break = to_pos!(ASTNode::Init);
    assert_eq!(desugar(&_break), Core::Id { lit: String::from("init") });
}

#[test]
fn self_verify() {
    let _break = to_pos!(ASTNode::_Self);
    assert_eq!(desugar(&_break), Core::Id { lit: String::from("self") });
}

#[test]
fn import_verify() {
    let _break = to_pos!(ASTNode::Import {
        import: vec![to_pos_unboxed!(ASTNode::Id { lit: String::from("a") })],
        _as:    vec![to_pos_unboxed!(ASTNode::Id { lit: String::from("b") })]
    });

    assert_eq!(desugar(&_break), Core::ImportAs {
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

    assert_eq!(desugar(&_break), Core::FromImport {
        from:   Box::from(Core::Id { lit: String::from("f") }),
        import: Box::from(Core::ImportAs {
            import: vec![Core::Id { lit: String::from("a") }],
            _as:    vec![Core::Id { lit: String::from("b") }]
        })
    });
}

#[test]
fn top_level_body_panic_verify() {
    let var_def = to_pos!(ASTNode::Body { isa: vec![], definitions: vec![] });

    panic::set_hook(Box::new(|_info| {}));
    let result = std::panic::catch_unwind(|| desugar(&var_def));
    assert!(result.is_err());
}

#[test]
fn raises_empty_verify() {
    let type_def = to_pos!(ASTNode::Raises {
        expr_or_stmt: Box::from(to_pos!(ASTNode::Pass)),
        errors:       vec![]
    });
    assert_eq!(desugar(&type_def), Core::Empty);
}
