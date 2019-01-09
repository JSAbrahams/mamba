use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::maybe_expr::parse_expression;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// reassignment     ::= maybe-expr "<-" maybe-expr
// assignment       ::= mutable-assign | immutable-assign
// mutable-assign   ::= [ "mutable" ] immutable-assignment
// immutable-assign ::= definition "<-" maybe-expr
// definition       ::= "let" id

pub fn parse_reassignment(pre: ASTNode, it: &mut Peekable<Iter<Token>>, ind: i32)
                          -> (Result<ASTNode, String>, i32) {
    if it.next() != Some(&Token::Assign) { return (Err("Expected '<-' keyword".to_string()), ind); }

    match parse_expression(it, ind) {
        (Ok(expr), ind) => (Ok(ASTNode::Assign(wrap!(pre), wrap!(expr))), ind),
        err => err
    }
}

pub fn parse_assignment(it: &mut Peekable<Iter<Token>>, ind: i32)
                        -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::Let) => pare_immutable_assign(it, ind),
        Some(Token::Mut) => parse_mutable_assign(it, ind),
        Some(_) | None => (Err("Expected assignment.".to_string()), ind)
    };
}

fn parse_mutable_assign(it: &mut Peekable<Iter<Token>>, ind: i32)
                        -> (Result<ASTNode, String>, i32) {
    if it.next() != Some(&Token::Mut) {
        return (Err("Expected 'mutable' keyword".to_string()), ind);
    }

    match pare_immutable_assign(it, ind) {
        (Ok(assign), ind) => (Ok(ASTNode::Mut(wrap!(assign))), ind),
        err => err
    }
}

fn pare_immutable_assign(it: &mut Peekable<Iter<Token>>, ind: i32)
                         -> (Result<ASTNode, String>, i32) {
    match parse_definition(it, ind) {
        (Ok(letid), ind) => if Some(&Token::Assign) == it.next() {
            match parse_expression(it, ind) {
                (Ok(expr), ind) => (Ok(ASTNode::Assign(wrap!(letid), wrap!(expr))), ind),
                err => err
            }
        } else {
            (Err("Expected assign operator".to_string()), ind)
        }
        err => err
    }
}

fn parse_definition(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    if it.next() != Some(&Token::Let) { return (Err("Expected 'let' keyword".to_string()), ind); }

    match it.next() {
        Some(Token::Id(id)) => (Ok(ASTNode::Let(wrap!(ASTNode::Id(id.to_string())))), ind),
        Some(_) | None => (Err("Expected definition.".to_string()), ind)
    }
}