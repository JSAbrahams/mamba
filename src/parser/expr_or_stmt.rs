use crate::lexer::token::Token;
use crate::parser::_type::parse_generics;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::block::parse_block;
use crate::parser::control_flow_expr::parse_match_cases;
use crate::parser::expression::parse_expression;
use crate::parser::iterator::TPIterator;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::statement::parse_statement;

pub fn parse_expr_or_stmt(it: &mut TPIterator) -> ParseResult {
    if it.eat_if_token(Token::NL) {
        return it.parse(&parse_block, "expression or statement");
    }

    let result = it.peek_or_err(
        &|it, token_pos| match token_pos.token {
            Token::Def
            | Token::Mut
            | Token::Print
            | Token::For
            | Token::While
            | Token::Retry
            | Token::Pass
            | Token::Raise
            | Token::With => parse_statement(it),
            _ => parse_expression(it)
        },
        CustomEOFErr { expected: String::from("expression or statement") }
    )?;

    it.peek(
        &|it, token_pos| match token_pos.token {
            Token::Raises => parse_raise(*result.clone(), it),
            Token::Handle => parse_handle(*result.clone(), it),
            _ => Ok(result.clone())
        },
        Ok(result.clone())
    )
}

pub fn parse_raise(expr_or_stmt: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    it.eat_token(Token::Raises);

    let errors = it.parse_vec(&parse_generics, "raises")?;
    let (en_line, en_pos) = match errors.last() {
        Some(stmt) => (stmt.en_line, stmt.en_pos),
        None => (st_line, st_pos)
    };

    let node = ASTNode::Raises { expr_or_stmt: Box::from(expr_or_stmt), errors };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

pub fn parse_handle(expr_or_stmt: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    it.eat_token(Token::Handle)?;
    it.eat_token(Token::NL)?;

    let cases = it.parse_vec(&parse_match_cases, "handle cases")?;
    let (en_line, en_pos) = match cases.last() {
        Some(stmt) => (stmt.en_line, stmt.en_pos),
        None => (st_line, st_pos)
    };

    let node = ASTNode::Handle { expr_or_stmt: Box::from(expr_or_stmt), cases };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}
