use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::call::parse_anon_fun;
use crate::parser::call::parse_call;
use crate::parser::call::parse_reassignment;
use crate::parser::collection::parse_collection;
use crate::parser::control_flow_expr::parse_cntrl_flow_expr;
use crate::parser::iterator::TPIterator;
use crate::parser::operation::parse_operation;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;

pub fn parse_expression(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    let result = it.peek(
        &|token_pos| match token_pos.token {
            Token::If | Token::Match => parse_cntrl_flow_expr(it),
            Token::LRBrack | Token::LSBrack | Token::LCBrack => parse_collection(it),
            Token::Ret => parse_return(it),
            Token::Underscore => parse_underscore(it),

            Token::_Self
            | Token::Real(_)
            | Token::Int(_)
            | Token::ENum(..)
            | Token::Str(_)
            | Token::Bool(_)
            | Token::Not
            | Token::Sqrt
            | Token::Add
            | Token::Id(_)
            | Token::Sub
            | Token::BOneCmpl => parse_operation(it),

            Token::BSlash => parse_anon_fun(it),

            _ => Err(CustomErr { expected: "expression".to_string(), actual: token_pos.clone() })
        },
        CustomEOFErr { expected: "expression".to_string() }
    );

    match result {
        Ok(res) => parse_post_expr(res, it),
        err => err
    }
}

fn parse_underscore(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    it.eat(Token::Underscore);
    let (en_line, en_pos) = it.end_pos()?;
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Underscore }))
}

fn parse_post_expr(pre: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let result = it.peek_or(
        &|token_pos| match token_pos {
            TokenPos { token: Token::QuestOr, st_line, st_pos } => {
                it.eat(Token::QuestOr);
                let right = it.parse(parse_expression, "question or")?;
                let (en_line, en_pos) = (right.en_line, right.en_pos);
                let node = ASTNode::QuestOr { left: Box::new(pre), right };
                Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
            }

            TokenPos { token: Token::Assign, .. } => parse_reassignment(pre, it),

            TokenPos { token: Token::LRBrack, .. } | TokenPos { token: Token::Point, .. } =>
                parse_call(pre, it),
            &tp if is_start_expression_exclude_unary(&tp) => parse_call(pre, it),
            _ => Err(CustomErr {
                expected: "post expression".to_string(),
                actual:   token_pos.clone()
            })
        },
        pre
    );

    match result {
        Ok(res) => parse_post_expr(res, it),
        err => err
    }
}

fn parse_return(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    it.eat(Token::Ret);

    if it.eat_if(Token::NL) {
        let (en_line, en_pos) = it.end_pos()?;
        return Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::ReturnEmpty });
    }

    let expr = it.parse(&parse_expression, "return expression");
    let (en_line, en_pos) = (expr.en_line, expr.en_pos);
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Return { expr } })
}

/// Excluding unary addition and subtraction
pub fn is_start_expression_exclude_unary(next: &TokenPos) -> bool {
    match next {
        TokenPos { token: Token::If, .. }
        | TokenPos { token: Token::Match, .. }
        | TokenPos { token: Token::LRBrack, .. }
        | TokenPos { token: Token::LSBrack, .. }
        | TokenPos { token: Token::LCBrack, .. }
        | TokenPos { token: Token::Underscore, .. }
        | TokenPos { token: Token::_Self, .. }
        | TokenPos { token: Token::Real(_), .. }
        | TokenPos { token: Token::Int(_), .. }
        | TokenPos { token: Token::ENum(..), .. }
        | TokenPos { token: Token::Str(_), .. }
        | TokenPos { token: Token::Bool(_), .. }
        | TokenPos { token: Token::Not, .. }
        | TokenPos { token: Token::Id(_), .. } => true,
        _ => false
    }
}

pub fn is_start_expression(next: &TokenPos) -> bool {
    let start_expr = is_start_expression_exclude_unary(next);
    start_expr || next.token == Token::Add || next.token == Token::Sub
}
