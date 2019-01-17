use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::block::parse_block;
use crate::parser::control_flow_expr::parse_cntrl_flow_expr;
use crate::parser::declaration::parse_reassignment;
use crate::parser::function::parse_function_anonymous;
use crate::parser::function::parse_function_call;
use crate::parser::function::parse_function_call_direct;
use crate::parser::operation::parse_operation;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use std::env;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

pub fn parse_expression(it: &mut Peekable<Iter<TokenPos>>) -> ParseResult {
    print_parse!(it, "expression");
    let mut tuple = false;

    return match match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::If }) |
        Some(TokenPos { line: _, pos: _, token: Token::Unless }) |
        Some(TokenPos { line: _, pos: _, token: Token::When }) => parse_cntrl_flow_expr(it),
        Some(TokenPos { line: _, pos: _, token: Token::NL }) => {
            it.next();
            check_next_is!(it, Token::Indent);
            parse_block(it)
        }
        Some(TokenPos { line: _, pos: _, token: Token::LPar }) => {
            tuple = true;
            parse_tuple(it)
        }
        Some(TokenPos { line: _, pos: _, token: Token::LBrack }) => parse_set_builder(it),
        Some(TokenPos { line: _, pos: _, token: Token::Ret }) => parse_return(it),
        Some(TokenPos { line: _, pos: _, token: Token::Real(_) }) |
        Some(TokenPos { line: _, pos: _, token: Token::Int(_) }) |
        Some(TokenPos { line: _, pos: _, token: Token::ENum(_, _) }) |
        Some(TokenPos { line: _, pos: _, token: Token::Id(_) }) |
        Some(TokenPos { line: _, pos: _, token: Token::Str(_) }) |
        Some(TokenPos { line: _, pos: _, token: Token::Bool(_) }) |
        Some(TokenPos { line: _, pos: _, token: Token::Not }) |
        Some(TokenPos { line: _, pos: _, token: Token::Add }) |
        Some(TokenPos { line: _, pos: _, token: Token::Sub }) => parse_operation(it),

        Some(&next) => Err(CustomErr { expected: "expression".to_string(), actual: next.clone() }),
        None => Err(CustomEOFErr { expected: "expression".to_string() })
    } {
        Ok(pre) => match it.peek() {
            Some(TokenPos { line: _, pos: _, token: Token::Assign }) =>
                parse_reassignment(pre, it),
            Some(TokenPos { line: _, pos: _, token: Token::LPar }) =>
                parse_function_call_direct(pre, it),
            Some(TokenPos { line: _, pos: _, token: Token::Point }) =>
                parse_function_call(pre, it),
            Some(TokenPos { line: _, pos: _, token: Token::To }) if tuple => {
                it.next();
                let right = get_or_err!(it, parse_function_anonymous, "anonymous function");
                Ok(ASTNode::Assign { left: Box::new(pre), right })
            }

            Some(_) | None => Ok(pre)
        }
        err => err
    };
}

fn parse_set_builder(it: &mut Peekable<Iter<TokenPos>>) -> ParseResult {
    print_parse!(it, "set builder");
    check_next_is!(it, Token::LBrack);

    let set = get_or_err!(it, parse_expression, "set builder");
    check_next_is!(it, Token::Ver);

    let mut conditions = Vec::new();
    match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::RBrack }) => (),
        _ => {
            let condition = get_or_err_direct!(it, parse_expression, "tuple");
            conditions.push(condition);
        }
    }

    while let Some(t) = it.peek() {
        match *t {
            TokenPos { line: _, pos: _, token: Token::RBrack } => break,
            TokenPos { line: _, pos: _, token: Token::Comma } => {
                it.next();
                let condition = get_or_err_direct!(it, parse_expression, "tuple");
                conditions.push(condition);
            }
            tp =>
                return Err(CustomErr { expected: "tuple element".to_string(), actual: tp.clone() })
        };
    }

    check_next_is!(it, Token::RBrack);
    return Ok(ASTNode::SetBuilder { set, conditions });
}

pub fn parse_tuple(it: &mut Peekable<Iter<TokenPos>>) -> ParseResult {
    print_parse!(it, "tuple");
    check_next_is!(it, Token::LPar);

    let mut elements = Vec::new();
    match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::RPar }) => (),
        _ => {
            let element = get_or_err_direct!(it, parse_expression, "tuple");
            elements.push(element);
        }
    }

    while let Some(t) = it.peek() {
        match *t {
            TokenPos { line: _, pos: _, token: Token::RPar } => break,
            TokenPos { line: _, pos: _, token: Token::Comma } => {
                it.next();
                let element = get_or_err_direct!(it, parse_expression, "tuple");
                elements.push(element);
            }
            tp =>
                return Err(CustomErr { expected: "tuple element".to_string(), actual: tp.clone() })
        };
    }

    check_next_is!(it, Token::RPar);
    return Ok(ASTNode::Tuple { elements });
}

fn parse_return(it: &mut Peekable<Iter<TokenPos>>) -> ParseResult {
    print_parse!(it, "return");
    check_next_is!(it, Token::Ret);

    if let Some(&&TokenPos { line: _, pos: _, token: Token::NL }) = it.peek() {
        return Ok(ASTNode::ReturnEmpty);
    }

    let expr = get_or_err!(it, parse_expression, "return");
    return Ok(ASTNode::Return { expr });
}
