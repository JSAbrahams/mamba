use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::ASTNode;
use crate::parser::ASTNodePos;
use crate::parser::expression::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::start_pos;
use crate::parser::statement::parse_statement;
use crate::parser::TPIterator;

pub fn parse_expr_or_stmt(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    let pre: Box<ASTNodePos> = match it.peek() {
        Some(TokenPos { token: Token::Def, .. }) |
        Some(TokenPos { token: Token::Mut, .. }) |
        Some(TokenPos { token: Token::Print, .. }) |
        Some(TokenPos { token: Token::For, .. }) |
        Some(TokenPos { token: Token::While, .. }) =>
            get_or_err!(it, parse_statement, "expression or statement"),

        _ => get_or_err!(it, parse_expression, "expression or statement")
    };

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

        _ => Ok(*pre)
    };
}
