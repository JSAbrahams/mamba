use crate::common::*;
use mamba::lexer::tokenize;
use mamba::parser::ast::ASTNode;
use mamba::parser::parse_direct;

use mamba::parser::parse;

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
        ASTNode::Script { statements, .. } => {
            _statements = statements;
            match &_statements.first().expect("script empty.").node {
                ASTNode::List { elements } => elements,
                _ => panic!("first element script was not list.")
            }
        }
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(elements[0].node, ASTNode::Id { lit: String::from("a") });
    assert_eq!(elements[1].node, ASTNode::Id { lit: String::from("b") });
}

#[test]
fn list_builder_verify() {
    let source = String::from("[a | c, d]");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let _statements;
    let (items, conditions) = match ast_tree.node {
        ASTNode::Script { statements, .. } => {
            _statements = statements;
            match &_statements.first().expect("script empty.").node {
                ASTNode::ListBuilder { items, conditions } => (items, conditions),
                _ => panic!("first element script was not list builder.")
            }
        }
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(items.node, ASTNode::Id { lit: String::from("a") });

    assert_eq!(conditions.len(), 2);
    assert_eq!(conditions[0].node, ASTNode::Id { lit: String::from("c") });
    assert_eq!(conditions[1].node, ASTNode::Id { lit: String::from("d") });
}

#[test]
fn parse_map() {
    let source = resource_content(true, &["collection"], "map.mamba");
    parse(&tokenize(&source).unwrap()).unwrap();
}

#[test]
fn set_verify() {
    let source = String::from("{a, b}");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let _statements;
    let elements = match ast_tree.node {
        ASTNode::Script { statements, .. } => {
            _statements = statements;
            match &_statements.first().expect("script empty.").node {
                ASTNode::Set { elements } => elements,
                _ => panic!("first element script was not set.")
            }
        }
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(elements[0].node, ASTNode::Id { lit: String::from("a") });
    assert_eq!(elements[1].node, ASTNode::Id { lit: String::from("b") });
}

#[test]
fn set_builder_verify() {
    let source = String::from("{a | c, d}");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let _statements;
    let (items, conditions) = match ast_tree.node {
        ASTNode::Script { statements, .. } => {
            _statements = statements;
            match &_statements.first().expect("script empty.").node {
                ASTNode::SetBuilder { items, conditions } => (items, conditions),
                _ => panic!("first element script was not set builder.")
            }
        }
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(items.node, ASTNode::Id { lit: String::from("a") });

    assert_eq!(conditions.len(), 2);
    assert_eq!(conditions[0].node, ASTNode::Id { lit: String::from("c") });
    assert_eq!(conditions[1].node, ASTNode::Id { lit: String::from("d") });
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
        ASTNode::Script { statements, .. } => {
            _statements = statements;
            match &_statements.first().expect("script empty.").node {
                ASTNode::Tuple { elements } => elements,
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
        ASTNode::Script { statements, .. } => {
            _statements = statements;
            match &_statements.first().expect("script empty.").node {
                ASTNode::Id { lit } => lit,
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
        ASTNode::Script { statements, .. } => {
            _statements = statements;
            match &_statements.first().expect("script empty.").node {
                ASTNode::Tuple { elements } => elements,
                _ => panic!("first element script was not tuple.")
            }
        }
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(elements.len(), 2);
    assert_eq!(elements[0].node, ASTNode::Id { lit: String::from("d") });
    assert_eq!(elements[1].node, ASTNode::Id { lit: String::from("c") });
}
