use std::panic;

use mamba::common::position::Position;
use mamba::core::construct::Core;
use mamba::desugar::desugar;
use mamba::parse::ast::AST;
use mamba::parse::ast::Node;

#[test]
fn break_verify() {
    let _break = to_pos!(Node::Break);
    assert_eq!(desugar(&_break).unwrap(), Core::Break);
}

#[test]
fn continue_verify() {
    let _continue = to_pos!(Node::Continue);
    assert_eq!(desugar(&_continue).unwrap(), Core::Continue);
}

#[test]
fn pass_verify() {
    let pass = to_pos!(Node::Pass);
    assert_eq!(desugar(&pass).unwrap(), Core::Pass);
}

#[test]
fn print_verify() {
    let expr = to_pos!(Node::Str { lit: String::from("a"), expressions: vec![] });
    let print_stmt = to_pos!(Node::Print { expr });
    assert_eq!(desugar(&print_stmt).unwrap(), Core::Print {
        expr: Box::from(Core::Str { string: String::from("a") })
    });
}

#[test]
fn return_verify() {
    let expr = to_pos!(Node::Str { lit: String::from("a"), expressions: vec![] });
    let print_stmt = to_pos!(Node::Return { expr });

    assert_eq!(desugar(&print_stmt).unwrap(), Core::Return {
        expr: Box::from(Core::Str { string: String::from("a") })
    });
}

#[test]
fn return_empty_verify() {
    let print_stmt = to_pos!(Node::ReturnEmpty);
    assert_eq!(desugar(&print_stmt).unwrap(), Core::Return { expr: Box::from(Core::None) });
}

#[test]
fn init_verify() {
    let _break = to_pos!(Node::Init);
    assert_eq!(desugar(&_break).unwrap(), Core::Id { lit: String::from("init") });
}

#[test]
fn self_verify() {
    let _break = to_pos!(Node::_Self);
    assert_eq!(desugar(&_break).unwrap(), Core::Id { lit: String::from("self") });
}

#[test]
fn import_verify() {
    let _break = to_pos!(Node::Import {
        import: vec![to_pos_unboxed!(Node::Id { lit: String::from("a") })],
        aliases:    vec![to_pos_unboxed!(Node::Id { lit: String::from("b") })]
    });

    assert_eq!(desugar(&_break).unwrap(), Core::ImportAs {
        imports: vec![Core::Id { lit: String::from("a") }],
        alias: vec![Core::Id { lit: String::from("b") }],
    });
}

#[test]
fn from_import_as_verify() {
    let _break = to_pos!(Node::FromImport {
        id:     to_pos!(Node::Id { lit: String::from("f") }),
        import: to_pos!(Node::Import {
            import: vec![to_pos_unboxed!(Node::Id { lit: String::from("a") })],
            aliases:    vec![to_pos_unboxed!(Node::Id { lit: String::from("b") })]
        })
    });

    assert_eq!(desugar(&_break).unwrap(), Core::FromImport {
        from: Box::from(Core::Id { lit: String::from("f") }),
        import: Box::from(Core::ImportAs {
            imports: vec![Core::Id { lit: String::from("a") }],
            alias: vec![Core::Id { lit: String::from("b") }],
        }),
    });
}

#[test]
fn raises_empty_verify() {
    let type_def = to_pos!(Node::Raises {
        expr_or_stmt: Box::from(to_pos!(Node::Id { lit: String::from("a") })),
        errors:       vec![]
    });
    assert_eq!(desugar(&type_def).unwrap(), Core::Id { lit: String::from("a") });
}
