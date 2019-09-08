use crate::lexer::token::Token;
use crate::parser::_type::parse_id_maybe_type;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::control_flow_stmt::parse_cntrl_flow_stmt;
use crate::parser::definition::parse_definition;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::expression::parse_expression;
use crate::parser::iterator::TPIterator;
use crate::parser::parse_result::expected_one_of;
use crate::parser::parse_result::ParseResult;

pub fn parse_statement(it: &mut TPIterator) -> ParseResult {
    it.peek_or_err(
        &|it, token_pos| match token_pos.token {
            Token::Print => {
                it.eat(&Token::Print, "statement")?;
                let expr = it.parse(&parse_expression, "statement", &token_pos.start)?;
                let node = ASTNode::Print { expr: expr.clone() };
                Ok(Box::from(ASTNodePos::new(&token_pos.start, &expr.position.end.clone(), node)))
            }
            Token::Pass => {
                let end = it.eat(&Token::Pass, "statement")?;
                Ok(Box::from(ASTNodePos::new(&token_pos.start, &end, ASTNode::Pass)))
            }
            Token::Retry => {
                let end = it.eat(&Token::Retry, "statement")?;
                Ok(Box::from(ASTNodePos::new(&token_pos.start, &end, ASTNode::Retry)))
            }
            Token::Raise => {
                it.eat(&Token::Raise, "statement")?;
                let error = it.parse(&parse_expression, "statement", &token_pos.start)?;
                let node = ASTNode::Raise { error: error.clone() };
                Ok(Box::from(ASTNodePos::new(&token_pos.start, &error.position.end, node)))
            }
            Token::Def => parse_definition(it),
            Token::With => parse_with(it),
            Token::For | Token::While => parse_cntrl_flow_stmt(it),
            _ => Err(expected_one_of(
                &[
                    Token::Print,
                    Token::Pass,
                    Token::Raise,
                    Token::Def,
                    Token::With,
                    Token::For,
                    Token::While
                ],
                token_pos,
                "statement"
            ))
        },
        &[
            Token::Print,
            Token::Pass,
            Token::Raise,
            Token::Def,
            Token::With,
            Token::For,
            Token::While
        ],
        "statement"
    )
}

pub fn parse_with(it: &mut TPIterator) -> ParseResult {
    let start = it.start_pos("with")?;
    it.eat(&Token::With, "with")?;
    let resource = it.parse(&parse_expression, "with", &start)?;
    let _as = it.parse_if(&Token::As, &parse_id_maybe_type, "with id", &start)?;
    let expr = it.parse(&parse_expr_or_stmt, "with", &start)?;

    let node = ASTNode::With { resource, _as, expr: expr.clone() };
    Ok(Box::from(ASTNodePos::new(&start, &expr.position.end, node)))
}

pub fn is_start_statement(tp: &Token) -> bool {
    match tp {
        Token::Def
        | Token::Mut
        | Token::Print
        | Token::For
        | Token::While
        | Token::Retry
        | Token::Pass
        | Token::Raise
        | Token::With => true,
        _ => false
    }
}
