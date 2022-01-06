use mamba::lex::tokenize;
use mamba::parse::ast::AST;
use mamba::parse::ast::Node;
use mamba::parse::parse;

use crate::common::*;
use crate::parse::util::parse_direct;

#[test]
fn for_statements() {
    let source = resource_content(true, &["control_flow"], "for_statements.mamba");
    parse(&tokenize(&source).unwrap()).unwrap();
}

#[test]
fn for_statement_verify() {
    let source = String::from("for a in c do d");
    let statements = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let (expr, collection, body) = match &statements.first().expect("script empty.").node {
        Node::For { expr, col, body } => (expr.clone(), col.clone(), body.clone()),
        _ => panic!("nameunion element script was not for.")
    };

    assert_eq!(expr.node, Node::Id { lit: String::from("a") });
    assert_eq!(collection.node, Node::Id { lit: String::from("c") });
    assert_eq!(body.node, Node::Id { lit: String::from("d") });
}

#[test]
fn for_range_step_verify() {
    let source = String::from("for a in c .. d step e do f");
    let statements = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let (expr, col, body) = match &statements.first().expect("script empty.").node {
        Node::For { expr, col, body } => (expr.clone(), col.clone(), body.clone()),
        _ => panic!("nameunion element script was not foreach.")
    };

    match col.node {
        Node::Range { from, to, inclusive, step } => {
            assert_eq!(from.node, Node::Id { lit: String::from("c") });
            assert_eq!(to.node, Node::Id { lit: String::from("d") });
            assert!(!inclusive);
            assert_eq!(step.clone().unwrap().node, Node::Id { lit: String::from("e") });
        }
        _ => panic!("Expected range")
    }

    assert_eq!(expr.node, Node::Id { lit: String::from("a") });
    assert_eq!(body.node, Node::Id { lit: String::from("f") });
}

#[test]
fn for_range_incl_verify() {
    let source = String::from("for a in c ..= d do f");
    let statements = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let (expr, col, body) = match &statements.first().expect("script empty.").node {
        Node::For { expr, col, body } => (expr.clone(), col.clone(), body.clone()),
        _ => panic!("nameunion element script was not foreach.")
    };

    match col.node {
        Node::Range { from, to, inclusive, step } => {
            assert_eq!(from.node, Node::Id { lit: String::from("c") });
            assert_eq!(to.node, Node::Id { lit: String::from("d") });
            assert!(inclusive);
            assert_eq!(step, None);
        }
        _ => panic!("Expected range")
    }

    assert_eq!(expr.node, Node::Id { lit: String::from("a") });
    assert_eq!(body.node, Node::Id { lit: String::from("f") });
}

#[test]
fn if_stmt() {
    let source = resource_content(true, &["control_flow"], "if.mamba");
    assert!(parse(&tokenize(&source).unwrap()).is_ok());
}

#[test]
fn if_verify() {
    let source = String::from("if a then c");
    let statements = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let (cond, then, el) = match &statements.first().expect("script empty.").node {
        Node::IfElse { cond, then, el } => (cond, then, el),
        _ => panic!("nameunion element script was not if.")
    };

    assert_eq!(cond.node, Node::Id { lit: String::from("a") });
    assert_eq!(then.node, Node::Id { lit: String::from("c") });
    assert_eq!(el.is_none(), true);
}

#[test]
fn if_with_block_verify() {
    let source = String::from("if a then\n    c\n    d");
    let statements = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let (cond, then, el) = match &statements.first().expect("script empty.").node {
        Node::IfElse { cond, then, el } => (cond.clone(), then.clone(), el.clone()),
        _ => panic!("nameunion element script was not if.")
    };

    assert_eq!(cond.node, Node::Id { lit: String::from("a") });
    assert_eq!(el.is_none(), true);

    let block = match then.node {
        Node::Block { statements } => statements,
        other => panic!("then of if was not block, was: {:?}", other)
    };

    assert_eq!(block.len(), 2);
    assert_eq!(block[0].node, Node::Id { lit: String::from("c") });
    assert_eq!(block[1].node, Node::Id { lit: String::from("d") });
}

#[test]
fn if_else_verify() {
    let source = String::from("if a then c else d");
    let statements = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let (cond, then, el) = match &statements.first().expect("script empty.").node {
        Node::IfElse { cond, then, el } => (cond, then, el),
        _ => panic!("nameunion element script was not if.")
    };

    assert_eq!(cond.node, Node::Id { lit: String::from("a") });
    assert_eq!(then.node, Node::Id { lit: String::from("c") });
    assert_eq!(el.as_ref().unwrap().node, Node::Id { lit: String::from("d") });
}

#[test]
fn match_statements() {
    let source = resource_content(true, &["control_flow"], "match.mamba");
    parse(&tokenize(&source).unwrap()).unwrap();
}

#[test]
fn match_verify() {
    let source = String::from("match a\n    a => b\n    c => d");
    let statements = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let (cond, cases) = match &statements.first().expect("script empty.").node {
        Node::Match { cond, cases } => (cond.clone(), cases.clone()),
        _ => panic!("nameunion element script was not match.")
    };

    assert_eq!(cond.node, Node::Id { lit: String::from("a") });

    assert_eq!(cases.len(), 2);
    let (cond1, expr1, cond2, expr2) = match (&cases[0], &cases[1]) {
        (
            AST { node: Node::Case { cond: cond1, body: expr1 }, .. },
            AST { node: Node::Case { cond: cond2, body: expr2 }, .. }
        ) => match (&cond1.node, &cond2.node) {
            (
                Node::ExpressionType { expr: cond1, .. },
                Node::ExpressionType { expr: cond2, .. }
            ) => (cond1, expr1, cond2, expr2),
            other => panic!("expected expression type: {:?}", other)
        },
        _ => panic!("Cases incorrect.")
    };

    assert_eq!(cond1.node, Node::Id { lit: String::from("a") });
    assert_eq!(expr1.node, Node::Id { lit: String::from("b") });
    assert_eq!(cond2.node, Node::Id { lit: String::from("c") });
    assert_eq!(expr2.node, Node::Id { lit: String::from("d") });
}

#[test]
fn while_statements() {
    let source = resource_content(true, &["control_flow"], "while.mamba");
    parse(&tokenize(&source).unwrap()).unwrap();
}

#[test]
fn while_verify() {
    let source = String::from("while a do d");
    let statements = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let (cond, body) = match &statements.first().expect("script empty.").node {
        Node::While { cond, body } => (cond.clone(), body.clone()),
        _ => panic!("nameunion element script was not while.")
    };

    assert_eq!(cond.node, Node::Id { lit: String::from("a") });
    assert_eq!(body.node, Node::Id { lit: String::from("d") });
}
