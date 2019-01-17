use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::ASTNodePos;
use crate::parser::end_pos;
use crate::parser::maybe_expr::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::start_pos;
use crate::parser::statement::parse_statement;
use crate::parser::TPIterator;
use std::env;

pub fn parse_expr_or_stmt(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    let function = match it.peek() {
        Some(TokenPos { token: Token::Let, .. }) |
        Some(TokenPos { token: Token::Mut, .. }) |
        Some(TokenPos { token: Token::Print, .. }) |
        Some(TokenPos { token: Token::For, .. }) |
        Some(TokenPos { token: Token::While, .. }) => parse_statement,
        _ => parse_expression
    };

    let pre = get_or_err!(it, function, "expression or statement");

    return match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::If }) => {
            it.next();
            let cond: Box<ASTNodePos> = get_or_err!(it, parse_expression, "post if");
            Ok(ASTNodePos {
                st_line,
                st_pos,
                en_line: cond.en_line,
                en_pos: cond.en_pos,
                node: ASTNode::If { cond, then: pre },
            })
        }
        Some(TokenPos { line: _, pos: _, token: Token::Unless }) => {
            it.next();
            let cond: Box<ASTNodePos> = get_or_err!(it, parse_expression, "post unless");
            Ok(ASTNodePos {
                st_line,
                st_pos,
                en_line: cond.en_line,
                en_pos: cond.en_pos,
                node: ASTNode::Unless { cond, then: pre },
            })
        }
        _ => Ok(*pre)
    };
}
