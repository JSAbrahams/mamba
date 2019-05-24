use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
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
    it.peek(
        &|token_pos| match token_pos {
            TokenPos { token: Token::Print, st_line, st_pos } => {
                it.eat(Token::Print);
                let expr = it.parse(&parse_expression, "print statement")?;
                let (en_line, en_pos) = (expr.en_line, expr.en_pos);
                let node = ASTNode::Print { expr };
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
            }
            TokenPos { token: Token::Pass, st_line, st_pos } => {
                let (en_line, en_pos) = it.end_pos()?;
                it.eat(Token::Pass);
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Pass }))
            }
            TokenPos { token: Token::Retry, st_line, st_pos } => {
                let (en_line, en_pos) = it.end_pos()?;
                it.eat(Token::Retry);
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Retry }))
            }
            TokenPos { token: Token::Raise, st_line, st_pos } => {
                let (en_line, en_pos) = it.end_pos()?;
                it.eat(Token::Raise);
                let error = it.parse(&parse_expression, "raise")?;
                let node = ASTNode::Raise { error };
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
            }

            TokenPos { token: Token::Def, st_line, st_pos } => parse_definition(it),
            TokenPos { token: Token::With, st_line, st_pos } => parse_with(it),

            TokenPos { token: Token::For, .. } | TokenPos { token: Token::While, .. } =>
                parse_cntrl_flow_stmt(it),

            Some(&next) =>
                Err(CustomErr { expected: "statement".to_string(), actual: next.clone() }),
        },
        CustomEOFErr { expected: "statement".to_string() }
    )
}

pub fn parse_with(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    it.eat(Token::With);
    let resource = it.parse(&parse_expression, "with resource")?;
    let _as = it.parse_if(Token::As, &parse_id_maybe_type, "with id")?;
    let expr = it.parse(parse_expr_or_stmt, "with body")?;

    let (en_line, en_pos) = (expr.en_line, expr.en_pos);
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::With { resource, _as, expr } })
}
