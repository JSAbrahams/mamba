use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::maybe_expr::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::statement::parse_statement;
use std::env;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

pub fn parse_expr_or_stmt(it: &mut Peekable<Iter<TokenPos>>) -> ParseResult<ASTNode> {
    print_parse!(it, "expression or statement");

    let fun = match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::Let }) |
        Some(TokenPos { line: _, pos: _, token: Token::Mut }) |
        Some(TokenPos { line: _, pos: _, token: Token::Print }) |
        Some(TokenPos { line: _, pos: _, token: Token::For }) |
        Some(TokenPos { line: _, pos: _, token: Token::While }) => parse_statement,
        _ => parse_expression
    };

    let pre = get_or_err!(it, fun, "expression or statement");
    return match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::If }) => {
            it.next();
            let cond = get_or_err!(it, parse_expression, "post if");
            Ok(ASTNode::If { cond, then: pre })
        }
        Some(TokenPos { line: _, pos: _, token: Token::Unless }) => {
            it.next();
            let cond = get_or_err!(it, parse_expression, "post unless");
            Ok(ASTNode::Unless { cond, then: pre })
        }
        _ => Ok(*pre)
    };
}
