use crate::lexer::Token as Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::maybe_expr::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// control-flow-stmt::= loop | while | for | "break" | "continue"
// while            ::= "while" maybe-expr "do" expr-or-stmt
// for              ::= "for" maybe-expr "in" maybe-expr "do" expr-or-stmt

pub fn parse_cntrl_flow_stmt(it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                             -> (ParseResult<ASTNode>, i32) {
    return match it.peek() {
        Some(TokenPos { line, pos, token: Token::While }) => parse_while(it, ind),
        Some(TokenPos { line, pos, token: Token::For }) => parse_for(it, ind),
        Some(TokenPos { line, pos, token: Token::Break }) =>
            next_and!(it, (Ok(ASTNode::Break), ind)),
        Some(TokenPos { line, pos, token: Token::Continue }) =>
            next_and!(it, (Ok(ASTNode::Continue), ind)),

        Some(&&actual @ TokenPos { ref line, ref pos, token })
        if (token != Token::While || token != Token::For) =>
            return (Err(TokenErr { expected: Token::While, actual }), ind),
        None => return (Err(EOFErr { expected: Token::While }), ind)
    };
}

fn parse_while(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    match it.next() {
        Some(&actual @ TokenPos { ref line, ref pos, token }) if *token != Token::While =>
            return (Err(TokenErr { expected: Token::While, actual }), ind),
        None => return (Err(EOFErr { expected: Token::While }), ind)
    }

    return match parse_expression(it, ind) {
        (Ok(cond), ind) => {
            match it.next() {
                Some(&actual @ TokenPos { ref line, ref pos, token }) if token != Token::Do =>
                    return (Err(TokenErr { expected: Token::Do, actual }), ind),
                None => return (Err(EOFErr { expected: Token::Do }), ind)
            }

            match parse_expr_or_stmt(it, ind) {
                (Ok(expr_or_do), ind) => (Ok(ASTNode::While(wrap!(cond), wrap!(expr_or_do))), ind),
                err => err
            }
        }
        err => err
    };
}

fn parse_for(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    match it.next() {
        Some(&actual @ TokenPos { ref line, ref pos, token }) if *token != Token::For =>
            return (Err(TokenErr { expected: Token::For, actual }), ind),
        None => return (Err(EOFErr { expected: Token::For }), ind)
    }

    return match parse_expression(it, ind) {
        (Ok(expr), ind) => {
            match it.next() {
                Some(&actual @ TokenPos { ref line, ref pos, token }) if *token != Token::In =>
                    return (Err(TokenErr { expected: Token::In, actual }), ind),
                None => return (Err(EOFErr { expected: Token::In }), ind)
            }

            match parse_expression(it, ind) {
                (Ok(col), ind) => {
                    match it.next() {
                        Some(&actual @ TokenPos { ref line, ref pos, token })
                        if *token != Token::Do =>
                            return (Err(TokenErr { expected: Token::Do, actual }), ind),
                        None => return (Err(EOFErr { expected: Token::Do }), ind)
                    }

                    match parse_expr_or_stmt(it, ind) {
                        (Ok(expr_or_do), ind) =>
                            (Ok(ASTNode::For(wrap!(expr), wrap!(col), wrap!(expr_or_do))), ind),
                        err => err
                    }
                }
                err => err
            }
        }
        err => err
    };
}
