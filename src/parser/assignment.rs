use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::maybe_expr::parse_expression;
use crate::parser::parse_result::ParseError;
use crate::parser::parse_result::ParseResult;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// reassignment     ::= maybe-expr "<-" maybe-expr
// assignment       ::= mutable-assign | immutable-assign
// mutable-assign   ::= [ "mutable" ] immutable-assignment
// immutable-assign ::= definition "<-" maybe-expr
// definition       ::= "let" id

pub fn parse_reassignment(pre: ASTNode, it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                          -> (ParseResult<ASTNode>, i32) {
    match it.next() {
        Some(tp @ TokenPos { line, pos, token }) if *token != Token::Assign =>
            return (Err(ParseError::TokenError(*tp, Token::From)), ind),
        None => return (Err(ParseError::EOFError(Token::From)), ind)
    }

    match parse_expression(it, ind) {
        (Ok(expr), ind) => (Ok(ASTNode::Assign(wrap!(pre), wrap!(expr))), ind),
        err => err
    }
}

pub fn parse_assignment(it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                        -> (ParseResult<ASTNode>, i32) {
    return match it.peek() {
        Some(TokenPos { line, pos, token: Token::Let }) => pare_immutable_assign(it, ind),
        Some(TokenPos { line, pos, token: Token::Mut }) => parse_mutable_assign(it, ind),
        Some(tp) => (Err(ParseError::TokenError(**tp, Token::Let)), ind),
        None => (Err(ParseError::EOFError(Token::Let)), ind)
    };
}

fn parse_mutable_assign(it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                        -> (ParseResult<ASTNode>, i32) {
    match it.next() {
        Some(tp @ TokenPos { line, pos, token }) if *token != Token::Mut =>
            return (Err(ParseError::TokenError(*tp, Token::Mut)), ind),
        None => return (Err(ParseError::EOFError(Token::Mut)), ind)
    }

    match pare_immutable_assign(it, ind) {
        (Ok(assign), ind) => (Ok(ASTNode::Mut(wrap!(assign))), ind),
        err => err
    }
}

fn pare_immutable_assign(it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                         -> (ParseResult<ASTNode>, i32) {
    match parse_definition(it, ind) {
        (Ok(letid), ind) => {
            match it.next() {
                Some(tp @ TokenPos { line, pos, token }) if *token != Token::Assign =>
                    return (Err(ParseError::TokenError(*tp, Token::Assign)), ind),
                None => return (Err(ParseError::EOFError(Token::Assign)), ind)
            }

            match parse_expression(it, ind) {
                (Ok(expr), ind) => (Ok(ASTNode::Assign(wrap!(letid), wrap!(expr))), ind),
                err => err
            }
        }
        err => err
    }
}

fn parse_definition(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    match it.next() {
        Some(tp @ TokenPos { line, pos, token }) if *token != Token::Let =>
            return (Err(ParseError::TokenError(*tp, Token::Let)), ind),
        None => return (Err(ParseError::EOFError(Token::Let)), ind)
    }

    match it.next() {
        Some(TokenPos { line, pos, token: Token::Id(id) }) =>
            (Ok(ASTNode::Let(wrap!(ASTNode::Id(id.to_string())))), ind),
        Some(tp) => (Err(ParseError::TokenError(*tp, Token::Let)), ind),
        None => (Err(ParseError::EOFError(Token::Let)), ind)
    }
}