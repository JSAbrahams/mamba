use crate::common::*;
use mamba::lexer::tokenize;
use mamba::parser::ast::ASTNode;
use mamba::parser::ast::ASTNodePos;
use mamba::parser::parse;

#[test]
fn for_statements() {
    let source = resource_content(true, &["control_flow"], "for_statements.mamba");
    parse(&tokenize(&source).unwrap()).unwrap();
}

#[test]
fn for_statement_verify() {
    let source = String::from("for a in c do d");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let (expr, collection, body) = match ast_tree.node {
        ASTNode::File { statements, .. } =>
            match &statements.first().expect("script empty.").node {
                ASTNode::For { expr, body } => match &expr.node {
                    ASTNode::In { left, right } => (left.clone(), right.clone(), body.clone()),
                    other => panic!("Expected in but was {:?}", other)
                },
                _ => panic!("first element script was not for.")
            },
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(expr.node, ASTNode::Id { lit: String::from("a") });
    assert_eq!(collection.node, ASTNode::Id { lit: String::from("c") });
    assert_eq!(body.node, ASTNode::Id { lit: String::from("d") });
}

#[test]
fn for_range_step_verify() {
    let source = String::from("for a in c .. d step e do f");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let (expr, body) = match ast_tree.node {
        ASTNode::File { statements, .. } =>
            match &statements.first().expect("script empty.").node {
                ASTNode::For { expr, body } => (expr.clone(), body.clone()),
                _ => panic!("first element script was not foreach.")
            },
        _ => panic!("ast_tree was not script.")
    };

    match expr.node {
        ASTNode::In { left, right } => {
            assert_eq!(left.node, ASTNode::Id { lit: String::from("a") });
            match &right.node {
                ASTNode::Range { from, to, inclusive, step } => {
                    assert_eq!(from.node, ASTNode::Id { lit: String::from("c") });
                    assert_eq!(to.node, ASTNode::Id { lit: String::from("d") });
                    assert!(!inclusive);
                    assert_eq!(step.clone().unwrap().node, ASTNode::Id { lit: String::from("e") });
                }
                _ => panic!("Expected range")
            }
        }
        other => panic!("Expected in but was {:?}", other)
    }

    assert_eq!(body.node, ASTNode::Id { lit: String::from("f") });
}

#[test]
fn for_range_incl_verify() {
    let source = String::from("for a in c ..= d do f");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let (expr, body) = match ast_tree.node {
        ASTNode::File { statements, .. } =>
            match &statements.first().expect("script empty.").node {
                ASTNode::For { expr, body } => (expr.clone(), body.clone()),
                _ => panic!("first element script was not foreach.")
            },
        _ => panic!("ast_tree was not script.")
    };

    match expr.node {
        ASTNode::In { left, right } => {
            assert_eq!(left.node, ASTNode::Id { lit: String::from("a") });
            match &right.node {
                ASTNode::Range { from, to, inclusive, step } => {
                    assert_eq!(from.node, ASTNode::Id { lit: String::from("c") });
                    assert_eq!(to.node, ASTNode::Id { lit: String::from("d") });
                    assert!(inclusive);
                    assert_eq!(*step, None);
                }
                _ => panic!("Expected range")
            }
        }
        other => panic!("Expected in but was {:?}", other)
    }

    assert_eq!(body.node, ASTNode::Id { lit: String::from("f") });
}

#[test]
fn if_stmt() {
    let source = resource_content(true, &["control_flow"], "if.mamba");
    assert!(parse(&tokenize(&source).unwrap()).is_ok());
}

#[test]
fn if_verify() {
    let source = String::from("if a then c");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let _statements;
    let (cond, then, _else) = match ast_tree.node {
        ASTNode::File { statements, .. } => {
            _statements = statements;
            match &_statements.first().expect("script empty.").node {
                ASTNode::IfElse { cond, then, _else } => (cond, then, _else),
                _ => panic!("first element script was not if.")
            }
        }
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(cond.node, ASTNode::Id { lit: String::from("a") });
    assert_eq!(then.node, ASTNode::Id { lit: String::from("c") });
    assert_eq!(_else.is_none(), true);
}

#[test]
fn if_with_block_verify() {
    let source = String::from("if a then\n    c\n    d");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let (cond, then, _else) = match ast_tree.node {
        ASTNode::File { statements, .. } =>
            match &statements.first().expect("script empty.").node {
                ASTNode::IfElse { cond, then, _else } =>
                    (cond.clone(), then.clone(), _else.clone()),
                _ => panic!("first element script was not if.")
            },
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(cond.node, ASTNode::Id { lit: String::from("a") });
    assert_eq!(_else.is_none(), true);

    let block = match then.node {
        ASTNode::Block { statements } => statements,
        other => panic!("then of if was not block, was: {:?}", other)
    };

    assert_eq!(block.len(), 2);
    assert_eq!(block[0].node, ASTNode::Id { lit: String::from("c") });
    assert_eq!(block[1].node, ASTNode::Id { lit: String::from("d") });
}

#[test]
fn if_else_verify() {
    let source = String::from("if a then c else d");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let _statements;
    let (cond, then, _else) = match ast_tree.node {
        ASTNode::File { statements, .. } => {
            _statements = statements;
            match &_statements.first().expect("script empty.").node {
                ASTNode::IfElse { cond, then, _else } => (cond, then, _else),
                _ => panic!("first element script was not if.")
            }
        }
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(cond.node, ASTNode::Id { lit: String::from("a") });
    assert_eq!(then.node, ASTNode::Id { lit: String::from("c") });
    assert_eq!(_else.as_ref().unwrap().node, ASTNode::Id { lit: String::from("d") });
}

#[test]
fn match_statements() {
    let source = resource_content(true, &["control_flow"], "match.mamba");
    assert!(parse(&tokenize(&source).unwrap()).is_ok());
}

#[test]
fn match_verify() {
    let source = String::from("match a\n    a => b\n    c => d");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let (cond, cases) = match ast_tree.node {
        ASTNode::File { statements, .. } =>
            match &statements.first().expect("script empty.").node {
                ASTNode::Match { cond, cases } => (cond.clone(), cases.clone()),
                _ => panic!("first element script was not match.")
            },
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(cond.node, ASTNode::Id { lit: String::from("a") });

    assert_eq!(cases.len(), 2);
    let (cond1, expr1, cond2, expr2) = match (&cases[0], &cases[1]) {
        (
            ASTNodePos { node: ASTNode::Case { cond: cond1, body: expr1 }, .. },
            ASTNodePos { node: ASTNode::Case { cond: cond2, body: expr2 }, .. }
        ) => match (&cond1.node, &cond2.node) {
            (ASTNode::IdType { id: cond1, .. }, ASTNode::IdType { id: cond2, .. }) =>
                (cond1, expr1, cond2, expr2),
            other => panic!("expected id maybe type: {:?}", other)
        },
        _ => panic!("Cases incorrect.")
    };

    assert_eq!(cond1.node, ASTNode::Id { lit: String::from("a") });
    assert_eq!(expr1.node, ASTNode::Id { lit: String::from("b") });
    assert_eq!(cond2.node, ASTNode::Id { lit: String::from("c") });
    assert_eq!(expr2.node, ASTNode::Id { lit: String::from("d") });
}

#[test]
fn while_statements() {
    let source = resource_content(true, &["control_flow"], "while.mamba");
    parse(&tokenize(&source).unwrap()).unwrap();
}

#[test]
fn while_verify() {
    let source = String::from("while a do d");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let (cond, body) = match ast_tree.node {
        ASTNode::File { statements, .. } =>
            match &statements.first().expect("script empty.").node {
                ASTNode::While { cond, body } => (cond.clone(), body.clone()),
                _ => panic!("first element script was not while.")
            },
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(cond.node, ASTNode::Id { lit: String::from("a") });
    assert_eq!(body.node, ASTNode::Id { lit: String::from("d") });
}
