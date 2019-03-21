use crate::util::*;
use mamba::lexer::tokenize;
use mamba::parser::ast::ASTNode;
use mamba::parser::parse;
use mamba::parser::parse_direct;

#[test]
fn parse_for_statement_check() {
    let source = String::from("foreach a in c do d");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    match ast_tree.node {
        ASTNode::Script { statements, .. } =>
            match &statements.first().expect("script empty.").node {
                ASTNode::For { expr, collection, body } => {
                    assert_eq!(expr[0].node, ASTNode::Id { lit: String::from("a") });
                    assert_eq!(collection.node, ASTNode::Id { lit: String::from("c") });
                    assert_eq!(body.node, ASTNode::Id { lit: String::from("d") });
                }
                _ => panic!("first element script was not for loop.")
            },
        _ => panic!("ast_tree was not script.")
    };
}

#[test]
fn parse_for_statement_multiple_expr_check() {
    let source = String::from("foreach a,b in c do d");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    match ast_tree.node {
        ASTNode::Script { statements, .. } =>
            match &statements.first().expect("script empty.").node {
                ASTNode::For { expr, collection, body } => {
                    assert_eq!(expr[0].node, ASTNode::Id { lit: String::from("a") });
                    assert_eq!(expr[1].node, ASTNode::Id { lit: String::from("b") });
                    assert_eq!(collection.node, ASTNode::Id { lit: String::from("c") });
                    assert_eq!(body.node, ASTNode::Id { lit: String::from("d") });
                }
                _ => panic!("first element script was not for loop.")
            },
        _ => panic!("ast_tree was not script.")
    };
}

#[test]
fn parse_for_statements() {
    let source = valid_resource_content(&["control_flow"], "for_statements.mamba");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_if() {
    let source = valid_resource_content(&["control_flow"], "if.mamba");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_match_statements() {
    let source = valid_resource_content(&["control_flow"], "match_statements.mamba");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_while_statements() {
    let source = valid_resource_content(&["control_flow"], "while_statements.mamba");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}
