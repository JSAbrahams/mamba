use crate::common::*;
use mamba::lex::tokenize;
use mamba::parse::ast::Node;
use mamba::parse::parse;
use mamba::parse::parse_direct;

#[test]
fn list_expression() {
    let source = resource_content(true, &["collection"], "tuple.mamba");
    parse(&tokenize(&source).unwrap()).unwrap();
}

#[test]
fn list_verify() {
    let source = String::from("[a, b]");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let _statements;
    let elements = match ast_tree.node {
        Node::Script { statements, .. } => {
            _statements = statements;
            match &_statements.first().expect("script empty.").node {
                Node::List { elements } => elements,
                _ => panic!("first element script was not list.")
            }
        }
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(elements[0].node, Node::Id { lit: String::from("a") });
    assert_eq!(elements[1].node, Node::Id { lit: String::from("b") });
}

#[test]
fn list_builder_verify() {
    let source = String::from("[a | c, d]");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let (items, conditions) = match ast_tree.node {
        Node::Script { statements, .. } => match &statements.first().expect("script empty.").node {
            Node::ListBuilder { item, conditions } => (item.clone(), conditions.clone()),
            _ => panic!("first element script was not list builder.")
        },
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(items.node, Node::Id { lit: String::from("a") });

    assert_eq!(conditions.len(), 2);
    assert_eq!(conditions[0].node, Node::Id { lit: String::from("c") });
    assert_eq!(conditions[1].node, Node::Id { lit: String::from("d") });
}

#[test]
#[ignore]
fn parse_map() {
    let source = resource_content(true, &["collection"], "map.mamba");
    parse(&tokenize(&source).unwrap()).unwrap();
}

#[test]
fn parse_set() {
    let source = resource_content(true, &["collection"], "set.mamba");
    assert!(parse(&tokenize(&source).unwrap()).is_ok());
}

#[test]
fn set_verify() {
    let source = String::from("{a, b}");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let _statements;
    let elements = match ast_tree.node {
        Node::Script { statements, .. } => {
            _statements = statements;
            match &_statements.first().expect("script empty.").node {
                Node::Set { elements } => elements,
                _ => panic!("first element script was not set.")
            }
        }
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(elements[0].node, Node::Id { lit: String::from("a") });
    assert_eq!(elements[1].node, Node::Id { lit: String::from("b") });
}

#[test]
fn set_builder_verify() {
    let source = String::from("{a | c, d}");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let (items, conditions) = match ast_tree.node {
        Node::Script { statements, .. } => match &statements.first().expect("script empty.").node {
            Node::SetBuilder { item, conditions } => (item.clone(), conditions.clone()),
            _ => panic!("first element script was not set builder.")
        },
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(items.node, Node::Id { lit: String::from("a") });

    assert_eq!(conditions.len(), 2);
    assert_eq!(conditions[0].node, Node::Id { lit: String::from("c") });
    assert_eq!(conditions[1].node, Node::Id { lit: String::from("d") });
}

#[test]
fn parse_tuple() {
    let source = resource_content(true, &["collection"], "tuple.mamba");
    parse(&tokenize(&source).unwrap()).unwrap();
}

#[test]
fn tuple_empty_verify() {
    let source = String::from("()");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let _statements;
    let elements = match ast_tree.node {
        Node::Script { statements, .. } => {
            _statements = statements;
            match &_statements.first().expect("script empty.").node {
                Node::Tuple { elements } => elements,
                _ => panic!("first element script was not tuple.")
            }
        }
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(elements.len(), 0);
}

#[test]
fn tuple_single_is_expr_verify() {
    let source = String::from("(a)");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let _statements;
    let lit = match ast_tree.node {
        Node::Script { statements, .. } => {
            _statements = statements;
            match &_statements.first().expect("script empty.").node {
                Node::Id { lit } => lit,
                _ => panic!("first element script was not tuple.")
            }
        }
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(lit.as_str(), "a");
}

#[test]
fn tuple_multiple_verify() {
    let source = String::from("(d, c)");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let _statements;
    let elements = match ast_tree.node {
        Node::Script { statements, .. } => {
            _statements = statements;
            match &_statements.first().expect("script empty.").node {
                Node::Tuple { elements } => elements,
                _ => panic!("first element script was not tuple.")
            }
        }
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(elements.len(), 2);
    assert_eq!(elements[0].node, Node::Id { lit: String::from("d") });
    assert_eq!(elements[1].node, Node::Id { lit: String::from("c") });
}
