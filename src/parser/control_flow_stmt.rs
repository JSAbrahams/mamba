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
        Some(TokenPos { line: _, pos: _, token: Token::While }) => parse_while(it, ind),
        Some(TokenPos { line: _, pos: _, token: Token::For }) => parse_for(it, ind),
        Some(TokenPos { line: _, pos: _, token: Token::Break }) =>
            next_and!(it, (Ok(ASTNode::Break), ind)),
        Some(TokenPos { line: _, pos: _, token: Token::Continue }) =>
            next_and!(it, (Ok(ASTNode::Continue), ind)),

        Some(&next) => return (Err(TokenErr { expected: Token::While, actual: next.clone() }), ind),
        None => return (Err(EOFErr { expected: Token::While }), ind)
    };
}

fn parse_while(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    check_next_is!(it, ind, Token::While);
    return match parse_expression(it, ind) {
        (Ok(cond), ind) => {
            check_next_is!(it, ind, Token::Do);
            match parse_expr_or_stmt(it, ind) {
                (Ok(expr_or_do), ind) => (Ok(ASTNode::While(get_or_err!(cond), get_or_err!(expr_or_do))), ind),
                err => err
            }
        }
        err => err
    };
}

fn parse_for(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    check_next_is!(it, ind, Token::For);

    return match parse_expression(it, ind) {
        (Ok(expr), ind) => {
            check_next_is!(it, ind, Token::In);
            match parse_expression(it, ind) {
                (Ok(col), ind) => {
                    check_next_is!(it, ind, Token::Do);
                    match parse_expr_or_stmt(it, ind) {
                        (Ok(expr_or_do), ind) =>
                            (Ok(ASTNode::For(get_or_err!(expr), get_or_err!(col), get_or_err!(expr_or_do))), ind),
                        err => err
                    }
                }
                err => err
            }
        }
        err => err
    };
}
