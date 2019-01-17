use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::ASTNodePos;
use crate::parser::block::parse_block;
use crate::parser::control_flow_expr::parse_cntrl_flow_expr;
use crate::parser::declaration::parse_reassignment;
use crate::parser::end_pos;
use crate::parser::function::parse_function_anonymous;
use crate::parser::function::parse_function_call;
use crate::parser::function::parse_function_call_direct;
use crate::parser::operation::parse_operation;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::start_pos;
use crate::parser::TPIterator;
use std::env;

pub fn parse_expression(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    let mut tuple = false;

    return match match it.peek() {
        Some(TokenPos { token: Token::If, .. }) |
        Some(TokenPos { token: Token::Unless, .. }) |
        Some(TokenPos { token: Token::When, .. }) => parse_cntrl_flow_expr(it),

        Some(TokenPos { token: Token::NL, .. }) => {
            it.next();
            check_next_is!(it, Token::Indent);
            parse_block(it)
        }

        Some(TokenPos { line: _, pos: _, token: Token::LPar }) => {
            tuple = true;
            parse_tuple(it)
        }

        Some(TokenPos { token: Token::LBrack, .. }) => parse_set_builder(it),
        Some(TokenPos { token: Token::Ret, .. }) => parse_return(it),

        Some(TokenPos { token: Token::Real(_), .. }) |
        Some(TokenPos { token: Token::Int(_), .. }) |
        Some(TokenPos { token: Token::ENum(_, _), .. }) |
        Some(TokenPos { token: Token::Id(_), .. }) |
        Some(TokenPos { token: Token::Str(_), .. }) |
        Some(TokenPos { token: Token::Bool(_), .. }) |
        Some(TokenPos { token: Token::Not, .. }) |
        Some(TokenPos { token: Token::Add, .. }) |
        Some(TokenPos { token: Token::Sub, .. }) => parse_operation(it),

        Some(&next) => Err(CustomErr { expected: "expression".to_string(), actual: next.clone() }),
        None => Err(CustomEOFErr { expected: "expression".to_string() })
    } {
        Ok(pre) => match it.peek() {
            Some(TokenPos { token: Token::Assign, .. }) => parse_reassignment(pre, it),
            Some(TokenPos { token: Token::LPar, .. }) => parse_function_call_direct(pre, it),
            Some(TokenPos { token: Token::Point, .. }) => parse_function_call(pre, it),
            Some(TokenPos { token: Token::To, .. }) if tuple => {
                it.next();
                let right: Box<ASTNodePos> = get_or_err!(it, parse_function_anonymous,
                                                         "anonymous function");
                Ok(ASTNodePos {
                    st_line,
                    st_pos,
                    en_line: right.en_line,
                    en_pos: right.en_pos,
                    node: ASTNode::Assign { left: Box::new(pre), right },
                })
            }
            Some(_) | None => Ok(pre)
        }
        err => err
    };
}

fn parse_set_builder(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::LBrack);

    let set = get_or_err!(it, parse_expression, "set builder");
    check_next_is!(it, Token::Ver);

    let mut conditions = Vec::new();
    match it.peek() {
        Some(TokenPos { token: Token::RBrack, .. }) => (),
        _ => {
            let condition = get_or_err_direct!(it, parse_expression, "tuple");
            conditions.push(condition);
        }
    }

    while let Some(t) = it.peek() {
        match *t {
            TokenPos { token: Token::RBrack, .. } => break,
            TokenPos { token: Token::Comma, .. } => {
                it.next();
                let condition = get_or_err_direct!(it, parse_expression, "tuple");
                conditions.push(condition);
            }
            tp =>
                return Err(CustomErr { expected: "tuple element".to_string(), actual: tp.clone() })
        };
    }

    check_next_is!(it, Token::RBrack);
    let (en_line, en_pos) = end_pos(it);

    return Ok(ASTNodePos {
        st_line,
        st_pos,
        en_line,
        en_pos,
        node: ASTNode::SetBuilder { set, conditions },
    });
}

pub fn parse_tuple(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::LPar);

    let mut elements = Vec::new();
    match it.peek() {
        Some(TokenPos { token: Token::RPar, .. }) => (),
        _ => {
            let element = get_or_err_direct!(it, parse_expression, "tuple");
            elements.push(element);
        }
    }

    while let Some(t) = it.peek() {
        match *t {
            TokenPos { token: Token::RPar, .. } => break,
            TokenPos { token: Token::Comma, .. } => {
                it.next();
                let element = get_or_err_direct!(it, parse_expression, "tuple");
                elements.push(element);
            }
            tp =>
                return Err(CustomErr { expected: "tuple element".to_string(), actual: tp.clone() })
        };
    }

    check_next_is!(it, Token::RPar);
    let (en_line, en_pos) = end_pos(it);

    return Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Tuple { elements } });
}

fn parse_return(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::Ret);

    if let Some(&&TokenPos { token: Token::NL, .. }) = it.peek() {
        let (en_line, en_pos) = end_pos(it);
        return Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::ReturnEmpty });
    }

    let expr: Box<ASTNodePos> = get_or_err!(it, parse_expression, "return");
    return Ok(ASTNodePos {
        st_line,
        st_pos,
        en_line: expr.en_line,
        en_pos: expr.en_pos,
        node: ASTNode::Return { expr },
    });
}
