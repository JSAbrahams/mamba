use crate::lexer::token::Token;
use crate::parser::_type::parse_generics;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::block::parse_block;
use crate::parser::control_flow_expr::parse_match_cases;
use crate::parser::expression::parse_expression;
use crate::parser::iterator::TPIterator;
use crate::parser::parse_result::ParseResult;
use crate::parser::statement::is_start_statement;
use crate::parser::statement::parse_statement;

pub fn parse_expr_or_stmt(it: &mut TPIterator) -> ParseResult {
    let result = it.peek_or_err(
        &|it, token_pos| match &token_pos.token {
            Token::NL => {
                it.eat(&Token::NL, "expression or statement")?;
                it.parse(
                    &parse_block,
                    "expression or statement",
                    token_pos.st_line,
                    token_pos.st_pos
                )
            }
            token =>
                if is_start_statement(token) {
                    parse_statement(it)
                } else {
                    parse_expression(it)
                },
        },
        &[],
        "expression or statement"
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
    let (st_line, st_pos) = it.start_pos("raise")?;
    it.eat(&Token::Raises, "raise")?;

    it.eat(&Token::LSBrack, "raise")?;
    let errors = it.parse_vec(&parse_generics, "raise", st_line, st_pos)?;
    it.eat(&Token::RSBrack, "raise")?;
    it.eat_if(&Token::RSBrack);
    let (en_line, en_pos) = match errors.last() {
        Some(stmt) => (stmt.en_line, stmt.en_pos),
        None => (st_line, st_pos)
    };

    let node = ASTNode::Raises { expr_or_stmt: Box::from(expr_or_stmt), errors };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

pub fn parse_handle(expr_or_stmt: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos("handle")?;
    it.eat(&Token::Handle, "handle")?;
    it.eat(&Token::NL, "handle")?;

    let cases = it.parse_vec(&parse_match_cases, "handle", st_line, st_pos)?;
    let (en_line, en_pos) = match cases.last() {
        Some(stmt) => (stmt.en_line, stmt.en_pos),
        None => (st_line, st_pos)
    };

    let node = ASTNode::Handle { expr_or_stmt: Box::from(expr_or_stmt), cases };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}
