use crate::lexer::token::Token;
use crate::parser::_type::parse_id_maybe_type;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::control_flow_stmt::parse_cntrl_flow_stmt;
use crate::parser::definition::parse_definition;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::expression::parse_expression;
use crate::parser::iterator::TPIterator;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;

pub fn parse_statement(it: &mut TPIterator) -> ParseResult {
    it.peek_or_err(
        &|it, token_pos| match token_pos.token {
            Token::Print => {
                it.eat(Token::Print, "statement")?;
                let expr = it.parse(&parse_expression, "print statement")?;
                let (st_line, st_pos) = (token_pos.st_line, token_pos.st_pos);
                let (en_line, en_pos) = (expr.en_line, expr.en_pos);
                let node = ASTNode::Print { expr };
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
            }
            Token::Pass => {
                let (st_line, st_pos) = (token_pos.st_line, token_pos.st_pos);
                let (en_line, en_pos) = it.eat(Token::Pass, "statement")?;
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Pass }))
            }
            Token::Retry => {
                let (st_line, st_pos) = (token_pos.st_line, token_pos.st_pos);
                let (en_line, en_pos) = it.eat(Token::Retry, "statement")?;
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Retry }))
            }
            Token::Raise => {
                it.eat(Token::Raise, "statement")?;
                let (st_line, st_pos) = (token_pos.st_line, token_pos.st_pos);
                let error = it.parse(&parse_expression, "raise")?;
                let (en_line, en_pos) = (error.en_line, error.en_pos);
                let node = ASTNode::Raise { error };
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
            }
            Token::Def => parse_definition(it),
            Token::With => parse_with(it),
            Token::For | Token::While => parse_cntrl_flow_stmt(it),
            _ => Err(CustomErr { expected: "statement".to_string(), actual: token_pos.clone() })
        },
        "statement"
    )
}

pub fn parse_with(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    it.eat(Token::With, "with")?;
    let resource = it.parse(&parse_expression, "with resource")?;
    let _as = it.parse_if(Token::As, &parse_id_maybe_type, "with id")?;
    let expr = it.parse(&parse_expr_or_stmt, "with body")?;

    let (en_line, en_pos) = (expr.en_line, expr.en_pos);
    let node = ASTNode::With { resource, _as, expr };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
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
