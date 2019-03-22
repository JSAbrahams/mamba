use crate::util::*;
use mamba::lexer::tokenize;
use mamba::parser::ast::ASTNode;
use mamba::parser::ast::ASTNodePos;
use mamba::parser::parse;
use mamba::parser::parse_direct;

#[test]
fn foreach_statements() {
    let source = valid_resource_content(&["control_flow"], "for_statements.mamba");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn foreach_statement_verify() {
    let source = String::from("foreach a in c do d");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let _statements;
    let (expr, collection, body) = match ast_tree.node {
        ASTNode::Script { statements, .. } => {
            _statements = statements;
            match &_statements.first().expect("script empty.").node {
                ASTNode::For { expr, collection, body } => (expr, collection, body),
                _ => panic!("first element script was not for loop.")
            }
        }
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(expr[0].node, ASTNode::Id { lit: String::from("a") });
    assert_eq!(collection.node, ASTNode::Id { lit: String::from("c") });
    assert_eq!(body.node, ASTNode::Id { lit: String::from("d") });
}

#[test]
fn foreach_tuple_statement_verify() {
    let source = String::from("foreach a,b in c do d");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let _statements;
    let (expr, collection, body) = match ast_tree.node {
        ASTNode::Script { statements, .. } => {
            _statements = statements;
            match &_statements.first().expect("script empty.").node {
                ASTNode::For { expr, collection, body } => (expr, collection, body),
                _ => panic!("first element script was not for loop.")
            }
        }
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(expr[0].node, ASTNode::Id { lit: String::from("a") });
    assert_eq!(expr[1].node, ASTNode::Id { lit: String::from("b") });
    assert_eq!(collection.node, ASTNode::Id { lit: String::from("c") });
    assert_eq!(body.node, ASTNode::Id { lit: String::from("d") });
}

#[test]
fn if_stmt() {
    let source = valid_resource_content(&["control_flow"], "if.mamba");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn if_verify() {
    let source = String::from("if a then c");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let _statements;
    let (cond, then, _else) = match ast_tree.node {
        ASTNode::Script { statements, .. } => {
            _statements = statements;
            match &_statements.first().expect("script empty.").node {
                ASTNode::IfElse { cond, then, _else } => (cond, then, _else),
                _ => panic!("first element script was not for loop.")
            }
        }
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(cond.len(), 1);
    assert_eq!(cond[0].node, ASTNode::Id { lit: String::from("a") });
    assert_eq!(then.node, ASTNode::Id { lit: String::from("c") });
    assert_eq!(_else.is_none(), true);
}

#[test]
fn if_else_verify() {
    let source = String::from("if a then c else d");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let _statements;
    let (cond, then, _else) = match ast_tree.node {
        ASTNode::Script { statements, .. } => {
            _statements = statements;
            match &_statements.first().expect("script empty.").node {
                ASTNode::IfElse { cond, then, _else } => (cond, then, _else),
                _ => panic!("first element script was not for loop.")
            }
        }
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(cond.len(), 1);
    assert_eq!(cond[0].node, ASTNode::Id { lit: String::from("a") });
    assert_eq!(then.node, ASTNode::Id { lit: String::from("c") });
    assert_eq!(_else.as_ref().unwrap().node, ASTNode::Id { lit: String::from("d") });
}

#[test]
fn if_tuple_verify() {
    let source = String::from("if a,b then c");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let _statements;
    let (cond, then, _else) = match ast_tree.node {
        ASTNode::Script { statements, .. } => {
            _statements = statements;
            match &_statements.first().expect("script empty.").node {
                ASTNode::IfElse { cond, then, _else } => (cond, then, _else),
                _ => panic!("first element script was not for loop.")
            }
        }
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(cond.len(), 2);
    assert_eq!(cond[0].node, ASTNode::Id { lit: String::from("a") });
    assert_eq!(cond[1].node, ASTNode::Id { lit: String::from("b") });
    assert_eq!(then.node, ASTNode::Id { lit: String::from("c") });
    assert_eq!(_else.is_none(), true);
}

#[test]
fn match_statements() {
    let source = valid_resource_content(&["control_flow"], "match_statements.mamba");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn match_verify() {
    let source = String::from("match a with\n    a => b\n    c => d");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let _statements;
    let (cond, cases) = match ast_tree.node {
        ASTNode::Script { statements, .. } => {
            _statements = statements;
            match &_statements.first().expect("script empty.").node {
                ASTNode::Match { cond, cases } => (cond, cases),
                _ => panic!("first element script was not for loop.")
            }
        }
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(cond.len(), 1);
    assert_eq!(cond[0].node, ASTNode::Id { lit: String::from("a") });

    assert_eq!(cases.len(), 2);
    let (cond1, expr1, cond2, expr2) = match (&cases[0], &cases[1]) {
        (
            ASTNodePos { node: ASTNode::Case { cond: cond1, expr: expr1 }, .. },
            ASTNodePos { node: ASTNode::Case { cond: cond2, expr: expr2 }, .. }
        ) => (cond1, expr1, cond2, expr2),
        _ => panic!("Cases incorrect.")
    };

    assert_eq!(cond1.node, ASTNode::Id { lit: String::from("a") });
    assert_eq!(expr1.node, ASTNode::Id { lit: String::from("b") });
    assert_eq!(cond2.node, ASTNode::Id { lit: String::from("c") });
    assert_eq!(expr2.node, ASTNode::Id { lit: String::from("d") });
}

#[test]
fn match_tuple_verify() {
    let source = String::from("match a,b with\n    c => d");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let _statements;
    let (cond, _) = match ast_tree.node {
        ASTNode::Script { statements, .. } => {
            _statements = statements;
            match &_statements.first().expect("script empty.").node {
                ASTNode::Match { cond, cases } => (cond, cases),
                _ => panic!("first element script was not for loop.")
            }
        }
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(cond.len(), 2);
    assert_eq!(cond[0].node, ASTNode::Id { lit: String::from("a") });
    assert_eq!(cond[1].node, ASTNode::Id { lit: String::from("b") });
}

#[test]
fn while_statements() {
    let source = valid_resource_content(&["control_flow"], "while_statements.mamba");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn while_verify() {
    let source = String::from("while a do d");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let _statements;
    let (cond, body) = match ast_tree.node {
        ASTNode::Script { statements, .. } => {
            _statements = statements;
            match &_statements.first().expect("script empty.").node {
                ASTNode::While { cond, body } => (cond, body),
                _ => panic!("first element script was not for loop.")
            }
        }
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(cond.len(), 1);
    assert_eq!(cond[0].node, ASTNode::Id { lit: String::from("a") });
    assert_eq!(body.node, ASTNode::Id { lit: String::from("d") });
}

#[test]
fn while_tuple_verify() {
    let source = String::from("while a,b do d");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let _statements;
    let (cond, body) = match ast_tree.node {
        ASTNode::Script { statements, .. } => {
            _statements = statements;
            match &_statements.first().expect("script empty.").node {
                ASTNode::While { cond, body } => (cond, body),
                _ => panic!("first element script was not for loop.")
            }
        }
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(cond.len(), 2);
    assert_eq!(cond[0].node, ASTNode::Id { lit: String::from("a") });
    assert_eq!(cond[1].node, ASTNode::Id { lit: String::from("b") });
    assert_eq!(body.node, ASTNode::Id { lit: String::from("d") });
}
