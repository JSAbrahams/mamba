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

pub fn parse_reassignment(pre: Box<ASTNode>, it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                          -> ParseResult<ASTNode> {
    check_next_is!(it, ind, Token::Assign);
    let (expr, ind) = get_or_err!(parse_expression(it, ind), "reassignment");
    return (Ok(ASTNode::Assign(pre, expr)), ind);
}

pub fn parse_defer_declaration(it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                               -> ParseResult<ASTNode> {
    let (declaration, ind) = get_or_err!(parse_declaration(it, ind), "defer declaration");

    let mut properties = Vec::new();
    while let Some(t) = it.peek() {
        match *t {
            TokenPos { line: _, pos: _, token: Token::NL } => break,
            TokenPos { line: _, pos: _, token: Token::Comma } => {
                it.next();
                let (prop, ind) = get_or_err!(parse_expression(it, ind), "defer declaration");
                properties.push(property);
            }
            next => return Err(TokenErr { expected: Token::Comma, actual: next.clone() })
        };
    }

    return (Ok(ASTNode::Defer(declaration, properties)), ind);
}

pub fn parse_declaration(it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                         -> (ParseResult<ASTNode>, i32) {
    return match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::Let }) =>
            pare_immutable_declaration(it, ind),
        Some(TokenPos { line: _, pos: _, token: Token::Mut }) => parse_mutable_declaration(it, ind),
        Some(&next) => (Err(TokenErr { expected: Token::Let, actual: next.clone() }), ind),
        None => (Err(EOFErr { expected: Token::Let }), ind)
    };
}

fn parse_mutable_declaration(it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                             -> (ParseResult<ASTNode>, i32) {
    check_next_is!(it, ind, Token::Mut);
    let (dec, ind) = get_or_err!(parse_immutable_declaration(it, ind), "immutable declaration");
    return (Ok(ASTNode::Mut(dec)), ind);
}

fn pare_immutable_declaration(it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                              -> (ParseResult<ASTNode>, i32) {
    let (let_id, ind) = get_or_err!(parse_definition(it, ind), "definition");
    check_next_is!(it, ind, Token::Assign);
    let (expr, ind) = get_or_err!(parse_expression(it, ind), "definition");
    return (Ok(ASTNode::Assign(let_id, expr)), ind);
}

fn parse_definition(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    check_next_is!(it, ind, Token::Let);
    match it.next() {
        Some(TokenPos { line: _, pos: _, token: Token::Id(id) }) => match it.peek() {
            Some(TokenPos { line: _, pos: _, token: Token::DoublePoint }) =>
                match (it.next(), it.next()) {
                    (_, Some(TokenPos { line: _, pos: _, token: Token::Id(id) })) =>
                        (Ok(ASTNode::Let(get_or_err!(ASTNode::Id(id.to_string())))), ind),
                    (_, Some(next)) => (Err(TokenErr {
                        expected: Token::Id(String::new()),
                        actual: next.clone(),
                    }), ind),
                    (_, None) => (Err(EOFErr { expected: Token::Id(String::new()) }), ind)
                }
            _ => (Ok(ASTNode::Let(get_or_err!(ASTNode::Id(id.to_string())))), ind)
        }
        Some(next) => (Err(TokenErr {
            expected: Token::Id(String::new()),
            actual: next.clone(),
        }), ind),
        None => (Err(EOFErr { expected: Token::Id(String::new()) }), ind)
    }
}
