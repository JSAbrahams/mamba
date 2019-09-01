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
use crate::parser::parse_result::expected_one_of;
use crate::parser::parse_result::ParseResult;

pub fn parse_expression(it: &mut TPIterator) -> ParseResult {
    let result = it.peek_or_err(
        &|it, token_pos| match token_pos.token {
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

            _ => Err(expected_one_of(
                &[
                    Token::If,
                    Token::Match,
                    Token::LRBrack,
                    Token::LSBrack,
                    Token::LCBrack,
                    Token::Ret,
                    Token::Underscore,
                    Token::_Self,
                    Token::Real(String::new()),
                    Token::Int(String::new()),
                    Token::ENum(String::new(), String::new()),
                    Token::Bool(true),
                    Token::Bool(false),
                    Token::Not,
                    Token::Sqrt,
                    Token::Add,
                    Token::Id(String::new()),
                    Token::Sub,
                    Token::BOneCmpl,
                    Token::BSlash
                ],
                token_pos,
                "expression"
            ))
        },
        &[
            Token::If,
            Token::Match,
            Token::LRBrack,
            Token::LSBrack,
            Token::LCBrack,
            Token::Ret,
            Token::Underscore,
            Token::_Self,
            Token::Real(String::new()),
            Token::Int(String::new()),
            Token::ENum(String::new(), String::new()),
            Token::Bool(true),
            Token::Bool(false),
            Token::Not,
            Token::Sqrt,
            Token::Add,
            Token::Id(String::new()),
            Token::Sub,
            Token::BOneCmpl,
            Token::BSlash
        ],
        "expression"
    );

    match result {
        Ok(res) => parse_post_expr(&res, it),
        err => err
    }
}

fn parse_underscore(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos("underscore")?;
    let (en_line, en_pos) = it.eat(&Token::Underscore, "underscore")?;
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Underscore }))
}

fn parse_post_expr(pre: &ASTNodePos, it: &mut TPIterator) -> ParseResult {
    it.peek(
        &|it, token_pos| match token_pos.token {
            Token::Question => {
                let (st_line, st_pos) = (token_pos.st_line, token_pos.st_pos);
                it.eat(&Token::Question, "postfix expression")?;
                let right = it.parse(&parse_expression, "postfix expression", st_line, st_pos)?;
                let (en_line, en_pos) = (right.en_line, right.en_pos);
                let node = ASTNode::Question { left: Box::new(pre.clone()), right };
                let res = ASTNodePos { st_line, st_pos, en_line, en_pos, node };
                parse_post_expr(&res, it)
            }
            Token::Assign => {
                let res = parse_reassignment(pre, it)?;
                parse_post_expr(&res, it)
            }
            Token::LRBrack | Token::Point => {
                let res = parse_call(pre, it)?;
                parse_post_expr(&res, it)
            }
            _ =>
                if is_start_expression_exclude_unary(token_pos) {
                    let res = parse_call(pre, it)?;
                    parse_post_expr(&res, it)
                } else {
                    Ok(Box::from(pre.clone()))
                },
        },
        Ok(Box::from(pre.clone()))
    )
}

fn parse_return(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos("return")?;
    it.eat(&Token::Ret, "return")?;

    if let Some((en_line, en_pos)) = it.eat_if(&Token::NL) {
        let node = ASTNode::ReturnEmpty;
        return Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }));
    }

    let expr = it.parse(&parse_expression, "return", st_line, st_pos)?;
    let (en_line, en_pos) = (expr.en_line, expr.en_pos);
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Return { expr } }))
}

/// Excluding unary addition and subtraction
pub fn is_start_expression_exclude_unary(tp: &TokenPos) -> bool {
    match tp.token {
        Token::If
        | Token::Match
        | Token::LRBrack
        | Token::LSBrack
        | Token::LCBrack
        | Token::Underscore
        | Token::BSlash
        | Token::_Self
        | Token::Real(_)
        | Token::Int(_)
        | Token::ENum(..)
        | Token::Str(_)
        | Token::Bool(_)
        | Token::Not
        | Token::Id(_) => true,
        _ => false
    }
}

pub fn is_start_expression(tp: &TokenPos) -> bool {
    let start_expr = is_start_expression_exclude_unary(tp);
    start_expr || tp.token == Token::Add || tp.token == Token::Sub
}
