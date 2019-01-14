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

pub fn parse_cntrl_flow_stmt(it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                             -> ParseResult<ASTNode> {
    return match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::While }) => parse_while(it, ind),
        Some(TokenPos { line: _, pos: _, token: Token::For }) => parse_for(it, ind),
        Some(TokenPos { line: _, pos: _, token: Token::Break }) =>
            next_and!(it, Ok((ASTNode::Break, ind))),
        Some(TokenPos { line: _, pos: _, token: Token::Continue }) =>
            next_and!(it, Ok((ASTNode::Continue, ind))),

        Some(&next) => return Err(CustomErr {
            expected: "control flow statement".to_string(),
            actual: next.clone(),
        }),
        None => return Err(CustomEOFErr { expected: "control flow statement".to_string() })
    };
}

fn parse_while(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<ASTNode> {
    check_next_is!(it, Token::While);

    let (cond, ind) = get_or_err!(it, ind, parse_expression, "while condition");
    check_next_is!(it, Token::Do);
    let (expr_or_do, ind) = get_or_err!(it, ind, parse_expr_or_stmt, "while body");
    return Ok((ASTNode::While(cond, expr_or_do), ind));
}

fn parse_for(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<ASTNode> {
    check_next_is!(it, Token::For);

    let (expr, ind) = get_or_err!(it, ind, parse_expression, "for expression");
    check_next_is!(it, Token::In);
    let (collection, ind) = get_or_err!(it, ind,  parse_expression, "for collection");
    check_next_is!(it, Token::Do);
    let (for_bod, ind) = get_or_err!(it, ind,  parse_expr_or_stmt, "for body");
    return Ok((ASTNode::For(expr, collection, for_bod), ind));
}
