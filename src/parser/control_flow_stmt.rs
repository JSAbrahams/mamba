use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::expression::parse_expression;
use crate::parser::iterator::TPIterator;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;

pub fn parse_cntrl_flow_stmt(it: &mut TPIterator) -> ParseResult {
    it.peek_or_err(
        &|it, token_pos| match token_pos {
            TokenPos { token: Token::While, .. } => parse_while(it),
            TokenPos { token: Token::For, .. } => parse_for(it),
            TokenPos { token: Token::Break, st_line, st_pos } => {
                let (st_line, st_pos) = (*st_line, *st_pos);
                let (en_line, en_pos) = it.end_pos()?;
                it.eat_token(Token::Break)?;
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Break }))
            }
            TokenPos { token: Token::Continue, st_line, st_pos } => {
                let (st_line, st_pos) = (*st_line, *st_pos);
                let (en_line, en_pos) = it.end_pos()?;
                it.eat_token(Token::Continue)?;
                let node = ASTNode::Continue;
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
            }
            _ => Err(CustomErr {
                expected: "control flow statement".to_string(),
                actual:   token_pos.clone()
            })
        },
        CustomEOFErr { expected: "control flow statement".to_string() }
    )
}

fn parse_while(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    it.eat_token(Token::While)?;
    let cond = it.parse(&parse_expression, "while condition")?;
    it.eat_token(Token::Do)?;
    let body = it.parse(&parse_expr_or_stmt, "while body")?;

    let (en_line, en_pos) = (body.en_line, body.en_pos);
    let node = ASTNode::While { cond, body };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

fn parse_for(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    it.eat_token(Token::For)?;
    let expr = it.parse(&parse_expression, "for expression")?;
    it.eat_token(Token::Do)?;
    let body = it.parse(&parse_expr_or_stmt, "for body")?;

    let (en_line, en_pos) = (body.en_line, body.en_pos);
    let node = ASTNode::For { expr, body };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}
