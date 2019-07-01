use crate::lexer::token::Token;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::expression::parse_expression;
use crate::parser::iterator::TPIterator;
use crate::parser::parse_result::expected_one_of;
use crate::parser::parse_result::ParseResult;

pub fn parse_cntrl_flow_stmt(it: &mut TPIterator) -> ParseResult {
    it.peek_or_err(
        &|it, token_pos| match token_pos.token {
            Token::While => parse_while(it),
            Token::For => parse_for(it),
            Token::Break => {
                let (st_line, st_pos) = (token_pos.st_line, token_pos.st_pos);
                let (en_line, en_pos) = it.eat(&Token::Break, "control flow statement")?;
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Break }))
            }
            Token::Continue => {
                let (st_line, st_pos) = (token_pos.st_line, token_pos.st_pos);
                let (en_line, en_pos) = it.eat(&Token::Continue, "control flow statement")?;
                let node = ASTNode::Continue;
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
            }
            _ => Err(expected_one_of(
                &[Token::While, Token::For, Token::Break, Token::Continue],
                token_pos,
                "control flow statement"
            ))
        },
        &[Token::While, Token::For, Token::Break, Token::Continue],
        "control flow statement"
    )
}

fn parse_while(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos("while")?;
    it.eat(&Token::While, "while")?;
    let cond = it.parse(&parse_expression)?;
    it.eat(&Token::Do, "while")?;
    let body = it.parse(&parse_expr_or_stmt)?;

    let (en_line, en_pos) = (body.en_line, body.en_pos);
    let node = ASTNode::While { cond, body };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

fn parse_for(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos("for")?;
    it.eat(&Token::For, "for")?;
    let expr = it.parse(&parse_expression)?;
    it.eat(&Token::Do, "for")?;
    let body = it.parse(&parse_expr_or_stmt)?;

    let (en_line, en_pos) = (body.en_line, body.en_pos);
    let node = ASTNode::For { expr, body };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}
