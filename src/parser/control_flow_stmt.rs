use crate::lexer::Token as Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::ASTNodePos;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::maybe_expr::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::TPIterator;
use std::env;

pub fn parse_cntrl_flow_stmt(it: &mut TPIterator) -> ParseResult {
    print_parse!(it, "control flow statement");

    return match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::While }) => parse_while(it),
        Some(TokenPos { line: _, pos: _, token: Token::For }) => parse_for(it),
        Some(TokenPos { ref line, pos, token: Token::Break }) => {
            it.next();
            Ok(ASTNode::Break)
        }
        Some(TokenPos { line, pos, token: Token::Continue }) => {
            it.next();
            Ok(ASTNode::Continue)
        }

        Some(&next) => return Err(CustomErr {
            expected: "control flow statement".to_string(),
            actual: next.clone(),
        }),
        None => return Err(CustomEOFErr { expected: "control flow statement".to_string() })
    };
}

fn parse_while(it: &mut TPIterator) -> ParseResult {
    print_parse!(it, "while");
    check_next_is!(it, Token::While);

    let cond = get_or_err!(it, parse_expression, "while condition");
    check_next_is!(it, Token::Do);
    let body = get_or_err!(it, parse_expr_or_stmt, "while body");
    return Ok(ASTNode::While { cond, body });
}

fn parse_for(it: &mut TPIterator) -> ParseResult {
    print_parse!(it, "for");
    check_next_is!(it, Token::For);

    let expr = get_or_err!(it, parse_expression, "for expression");
    check_next_is!(it, Token::In);
    let collection = get_or_err!(it, parse_expression, "for collection");
    check_next_is!(it, Token::Do);
    let body = get_or_err!(it, parse_expr_or_stmt, "for body");

    return Ok(ASTNode::For { expr, collection, body });
}
