use std::panic;

use mamba::common::position::Position;
use mamba::core::construct::Core;
use mamba::desugar::desugar;
use mamba::parse::ast::Node;
use mamba::parse::ast::AST;

#[test]
fn with_verify() {
    let resource = to_pos!(Node::Id { lit: String::from("my_resource") });
    let alias = Some((to_pos!(Node::Id { lit: String::from("other") }), false, None));
    let expr = to_pos!(Node::Int { lit: String::from("9") });
    let with = to_pos!(Node::With { resource, alias, expr });

    let (resource, alias, expr) = match desugar(&with) {
        Ok(Core::WithAs { resource, alias, expr }) => (resource, alias, expr),
        other => panic!("Expected with as but was {:?}", other)
    };

    assert_eq!(*resource, Core::Id { lit: String::from("my_resource") });
    assert_eq!(*alias, Core::Id { lit: String::from("other") });
    assert_eq!(*expr, Core::Int { int: String::from("9") });
}

#[test]
fn with_no_as_verify() {
    let resource = to_pos!(Node::Id { lit: String::from("other") });
    let expr = to_pos!(Node::Int { lit: String::from("2341") });
    let with = to_pos!(Node::With { resource, alias: None, expr });

    let (resource, expr) = match desugar(&with) {
        Ok(Core::With { resource, expr }) => (resource, expr),
        other => panic!("Expected with but was {:?}", other)
    };

    assert_eq!(*resource, Core::Id { lit: String::from("other") });
    assert_eq!(*expr, Core::Int { int: String::from("2341") });
}

#[test]
fn handle_empty_verify() {
    let expr_or_stmt = to_pos!(Node::Id { lit: String::from("my_fun") });
    let handle = to_pos!(Node::Handle { expr_or_stmt, cases: vec![] });

    let (setup, _try, except) = match desugar(&handle) {
        Ok(Core::TryExcept { setup, attempt, except }) =>
            (setup.clone(), attempt.clone(), except.clone()),
        other => panic!("Expected try except but was {:?}", other)
    };

    assert_eq!(setup, None);
    assert_eq!(*_try, Core::Id { lit: String::from("my_fun") });
    assert!(except.is_empty());
}

#[test]
fn handle_verify() {
    let expr_or_stmt = to_pos!(Node::Id { lit: String::from("my_fun") });
    let cond = to_pos!(Node::ExpressionType {
        expr:    to_pos!(Node::Id { lit: String::from("err") }),
        mutable: false,
        ty:      Some(to_pos!(Node::Type {
            id:       to_pos!(Node::Id { lit: String::from("my_type") }),
            generics: vec![]
        }))
    });
    let body = to_pos!(Node::Int { lit: String::from("9999") });
    let case = to_pos_unboxed!(Node::Case { cond, body });
    let handle = to_pos!(Node::Handle { expr_or_stmt, cases: vec![case] });

    let (setup, _try, except) = match desugar(&handle) {
        Ok(Core::TryExcept { setup, attempt, except }) =>
            (setup.clone(), attempt.clone(), except.clone()),
        other => panic!("Expected try except but was {:?}", other)
    };

    assert_eq!(setup, None);
    assert_eq!(*_try, Core::Id { lit: String::from("my_fun") });
    assert_eq!(except.len(), 1);
    match &except[0] {
        Core::Except { id, class, body } => {
            assert_eq!(*id, Box::from(Core::Id { lit: String::from("err") }));
            assert_eq!(
                *class,
                Some(Box::from(Core::Type { lit: String::from("my_type"), generics: vec![] }))
            );
            assert_eq!(*body, Box::from(Core::Int { int: String::from("9999") }));
        }
        other => panic!("Expected except case but was {:?}", other)
    }
}
