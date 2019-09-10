use mamba::common::position::EndPoint;
use mamba::common::position::Position;
use mamba::core::construct::Core;
use mamba::desugar::desugar;
use mamba::parser::ast::Node;
use mamba::parser::ast::AST;
use std::panic;

#[test]
fn with_verify() {
    let resource = to_pos!(ASTNode::Id { lit: String::from("my_resource") });
    let _as = Some(to_pos!(ASTNode::Id { lit: String::from("other") }));
    let expr = to_pos!(ASTNode::Int { lit: String::from("9") });
    let with = to_pos!(ASTNode::With { resource, _as, expr });

    let (resource, _as, expr) = match desugar(&with) {
        Ok(Core::WithAs { resource, _as, expr }) => (resource, _as, expr),
        other => panic!("Expected with as but was {:?}", other)
    };

    assert_eq!(*resource, Core::Id { lit: String::from("my_resource") });
    assert_eq!(*_as, Core::Id { lit: String::from("other") });
    assert_eq!(*expr, Core::Int { int: String::from("9") });
}

#[test]
fn with_no_as_verify() {
    let resource = to_pos!(ASTNode::Id { lit: String::from("other") });
    let expr = to_pos!(ASTNode::Int { lit: String::from("2341") });
    let with = to_pos!(ASTNode::With { resource, _as: None, expr });

    let (resource, expr) = match desugar(&with) {
        Ok(Core::With { resource, expr }) => (resource, expr),
        other => panic!("Expected with but was {:?}", other)
    };

    assert_eq!(*resource, Core::Id { lit: String::from("other") });
    assert_eq!(*expr, Core::Int { int: String::from("2341") });
}

#[test]
fn handle_empty_verify() {
    let expr_or_stmt = to_pos!(ASTNode::Id { lit: String::from("my_fun") });
    let handle = to_pos!(ASTNode::Handle { expr_or_stmt, cases: vec![] });

    let (_try, except) = match desugar(&handle) {
        Ok(Core::Block { statements }) => {
            assert_eq!(statements.len(), 1);
            match &statements[0] {
                Core::TryExcept { _try, except } => (_try.clone(), except.clone()),
                other => panic!("Expected try except but was {:?}", other)
            }
        }
        other => panic!("Expected block but was {:?}", other)
    };

    assert_eq!(*_try, Core::Id { lit: String::from("my_fun") });
    assert!(except.is_empty());
}

#[test]
fn handle_verify() {
    let expr_or_stmt = to_pos!(ASTNode::Id { lit: String::from("my_fun") });
    let cond = to_pos!(ASTNode::IdType {
        id:      to_pos!(ASTNode::Id { lit: String::from("err") }),
        mutable: false,
        _type:   Some(to_pos!(ASTNode::Type {
            id:       to_pos!(ASTNode::Id { lit: String::from("my_type") }),
            generics: vec![]
        }))
    });
    let body = to_pos!(ASTNode::Int { lit: String::from("9999") });
    let case = to_pos_unboxed!(ASTNode::Case { cond, body });
    let handle = to_pos!(ASTNode::Handle { expr_or_stmt, cases: vec![case] });

    let (_try, except) = match desugar(&handle) {
        Ok(Core::Block { statements }) => {
            assert_eq!(statements.len(), 1);
            match &statements[0] {
                Core::TryExcept { _try, except } => (_try.clone(), except.clone()),
                other => panic!("Expected try except but was {:?}", other)
            }
        }
        other => panic!("Expected block but was {:?}", other)
    };

    assert_eq!(*_try, Core::Id { lit: String::from("my_fun") });
    assert_eq!(except.len(), 1);
    match &except[0] {
        Core::Except { id, class, body } => {
            assert_eq!(**id, Core::Id { lit: String::from("err") });
            assert_eq!(**class, Core::Id { lit: String::from("my_type") });
            assert_eq!(**body, Core::Int { int: String::from("9999") });
        }
        other => panic!("Expected except case but was {:?}", other)
    }
}
