use crate::lexer::Token as Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::maybe_expr::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use std::env;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

pub fn parse_cntrl_flow_stmt(it: &mut Peekable<Iter<TokenPos>>) -> ParseResult<ASTNode> {
    print_parse!(it, "control flow statement");

    return match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::While }) => parse_while(it),
        Some(TokenPos { line: _, pos: _, token: Token::For }) => parse_for(it),
        Some(TokenPos { line: _, pos: _, token: Token::Break }) =>
            next_and!(it, Ok(ASTNode::Break)),
        Some(TokenPos { line: _, pos: _, token: Token::Continue }) =>
            next_and!(it, Ok(ASTNode::Continue)),

        Some(&next) => return Err(CustomErr {
            expected: "control flow statement".to_string(),
            actual: next.clone(),
        }),
        None => return Err(CustomEOFErr { expected: "control flow statement".to_string() })
    };
}

fn parse_while(it: &mut Peekable<Iter<TokenPos>>) -> ParseResult<ASTNode> {
    print_parse!(it, "while");
    check_next_is!(it, Token::While);

    let cond = get_or_err!(it, parse_expression, "while condition");
    check_next_is!(it, Token::Do);
    let body = get_or_err!(it, parse_expr_or_stmt, "while body");
    return Ok(ASTNode::While { cond, body });
}

fn parse_for(it: &mut Peekable<Iter<TokenPos>>) -> ParseResult<ASTNode> {
    print_parse!(it, "for");
    check_next_is!(it, Token::For);

    let expr = get_or_err!(it, parse_expression, "for expression");
    check_next_is!(it, Token::In);
    let collection = get_or_err!(it, parse_expression, "for collection");
    check_next_is!(it, Token::Do);
    let body = get_or_err!(it, parse_expr_or_stmt, "for body");

    return Ok(ASTNode::For { expr, collection, body });
}
