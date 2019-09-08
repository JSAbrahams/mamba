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
                let end = it.eat(&Token::Break, "control flow statement")?;
                Ok(Box::from(ASTNodePos::new(&token_pos.start, &end, ASTNode::Break)))
            }
            Token::Continue => {
                let end = it.eat(&Token::Continue, "control flow statement")?;
                Ok(Box::from(ASTNodePos::new(&token_pos.start, &end, ASTNode::Continue)))
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
    let start = it.start_pos("while statement")?;
    it.eat(&Token::While, "while statement")?;
    let cond = it.parse(&parse_expression, "while statement", &start)?;
    it.eat(&Token::Do, "while")?;
    let body = it.parse(&parse_expr_or_stmt, "while statement", &start)?;

    let node = ASTNode::While { cond, body: body.clone() };
    Ok(Box::from(ASTNodePos::new(&start, &body.position.end, node)))
}

fn parse_for(it: &mut TPIterator) -> ParseResult {
    let start = it.start_pos("for statement")?;
    it.eat(&Token::For, "for statement")?;
    let expr = it.parse(&parse_expression, "for statement", &start)?;
    it.eat(&Token::Do, "for statement")?;
    let body = it.parse(&parse_expr_or_stmt, "for statement", &start)?;

    let node = ASTNode::For { expr, body: body.clone() };
    Ok(Box::from(ASTNodePos::new(&start, &body.position.end, node)))
}
