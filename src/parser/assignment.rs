use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::maybe_expr::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// reassignment     ::= maybe-expr "<-" maybe-expr
// defer-declaration::= declaration [ "forward" id { "," id } ]
// assignment       ::= mutable-assign | immutable-assign
// mutable-assign   ::= [ "mutable" ] immutable-assignment
// immutable-assign ::= definition "<-" maybe-expr
// definition       ::= "let" id [ ":" id ]

pub fn parse_reassignment(pre: ASTNode, it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                          -> (ParseResult<ASTNode>, i32) {
    check_next_is!(it, ind, Token::Assign);
    return match parse_expression(it, ind) {
        (Ok(expr), ind) => (Ok(ASTNode::Assign(wrap!(pre), wrap!(expr))), ind),
        err => err
    };
}

pub fn parse_defer_declaration(it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                               -> (ParseResult<ASTNode>, i32) {
    match parse_declaration(it, ind) {
        (Ok(declaration), ind) => {
            let mut properties = Vec::new();
            while let Some(t) = it.peek() {
                match *t {
                    TokenPos { line: _, pos: _, token: Token::NL } => break,
                    TokenPos { line: _, pos: _, token: Token::Comma } =>
                        match (it.next(), parse_expression(it, ind)) {
                            (_, (Ok(property), _)) => properties.push(property),
                            (_, (err, ind)) => return (err, ind)
                        }
                    next => return (Err(TokenErr { expected: Token::Comma, actual: next.clone() }), ind)
                };
            }

            (Ok(ASTNode::Defer(wrap!(declaration), properties)), ind)
        }
        err => err
    }
}

pub fn parse_declaration(it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                         -> (ParseResult<ASTNode>, i32) {
    return match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::Let }) => pare_immutable_declaration(it, ind),
        Some(TokenPos { line: _, pos: _, token: Token::Mut }) => parse_mutable_declaration(it, ind),
        Some(&next) => (Err(TokenErr { expected: Token::Let, actual: next.clone() }), ind),
        None => (Err(EOFErr { expected: Token::Let }), ind)
    };
}

fn parse_mutable_declaration(it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                             -> (ParseResult<ASTNode>, i32) {
    check_next_is!(it, ind, Token::Mut);
    match pare_immutable_declaration(it, ind) {
        (Ok(assign), ind) => (Ok(ASTNode::Mut(wrap!(assign))), ind),
        err => err
    }
}

fn pare_immutable_declaration(it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                              -> (ParseResult<ASTNode>, i32) {
    match parse_definition(it, ind) {
        (Ok(let_id), ind) => {
            check_next_is!(it, ind, Token::Assign);
            match parse_expression(it, ind) {
                (Ok(expr), ind) => (Ok(ASTNode::Assign(wrap!(let_id), wrap!(expr))), ind),
                err => err
            }
        }
        err => err
    }
}

fn parse_definition(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    check_next_is!(it, ind, Token::Let);
    match it.next() {
        Some(TokenPos { line: _, pos: _, token: Token::Id(id) }) => match it.peek() {
            Some(TokenPos { line: _, pos: _, token: Token::DoublePoint }) =>
                match (it.next(), it.next()) {
                    (_, Some(TokenPos { line: _, pos: _, token: Token::Id(id) })) =>
                        (Ok(ASTNode::Let(wrap!(ASTNode::Id(id.to_string())))), ind),
                    (_, Some(next)) => (Err(TokenErr {
                        expected: Token::Id(String::new()),
                        actual: next.clone(),
                    }), ind),
                    (_, None) => (Err(EOFErr { expected: Token::Id(String::new()) }), ind)
                }
            _ => (Ok(ASTNode::Let(wrap!(ASTNode::Id(id.to_string())))), ind)
        }
        Some(next) => (Err(TokenErr { expected: Token::Id(String::new()), actual: next.clone() }),
                       ind),
        None => (Err(EOFErr { expected: Token::Id(String::new()) }), ind)
    }
}
