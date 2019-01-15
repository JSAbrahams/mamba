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
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

pub fn parse_expression(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<ASTNode> {
    let mut tuple = false;

    return match match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::If }) |
        Some(TokenPos { line: _, pos: _, token: Token::Unless }) |
        Some(TokenPos { line: _, pos: _, token: Token::When }) => parse_cntrl_flow_expr(it, ind),
        Some(TokenPos { line: _, pos: _, token: Token::NL }) => {
            it.next();
            parse_block(it, ind + 1)
        }
        Some(TokenPos { line: _, pos: _, token: Token::LPar }) => {
            tuple = true;
            parse_tuple(it, ind)
        }
        Some(TokenPos { line: _, pos: _, token: Token::LBrack }) => parse_set_builder(it, ind),
        Some(TokenPos { line: _, pos: _, token: Token::Ret }) => parse_return(it, ind),
        Some(TokenPos { line: _, pos: _, token: Token::Real(_) }) |
        Some(TokenPos { line: _, pos: _, token: Token::Int(_) }) |
        Some(TokenPos { line: _, pos: _, token: Token::ENum(_, _) }) |
        Some(TokenPos { line: _, pos: _, token: Token::Id(_) }) |
        Some(TokenPos { line: _, pos: _, token: Token::Str(_) }) |
        Some(TokenPos { line: _, pos: _, token: Token::Bool(_) }) |
        Some(TokenPos { line: _, pos: _, token: Token::Not }) |
        Some(TokenPos { line: _, pos: _, token: Token::Add }) |
        Some(TokenPos { line: _, pos: _, token: Token::Sub }) => parse_operation(it, ind),

        Some(&next) => Err(CustomErr { expected: "expression".to_string(), actual: next.clone() }),
        None => Err(CustomEOFErr { expected: "expression".to_string() })
    } {
        Ok((pre, ind)) => match it.peek() {
            Some(TokenPos { line: _, pos: _, token: Token::Assign }) =>
                parse_reassignment(pre, it, ind),
            Some(TokenPos { line: _, pos: _, token: Token::LPar }) =>
                parse_function_call_direct(pre, it, ind),
            Some(TokenPos { line: _, pos: _, token: Token::Point }) =>
                parse_function_call(pre, it, ind),
            Some(TokenPos { line: _, pos: _, token: Token::To }) if tuple => {
                it.next();
                let (fun, ind) = get_or_err!(it, ind, parse_function_anonymous,
                                             "anonymous function");
                Ok((ASTNode::Assign(Box::new(pre), fun), ind))
            }

            Some(_) | None => Ok((pre, ind))
        }
        err => err
    };
}

// set-builder ::= "[" maybe-expr "| maybe-expr { "," maybe-expr } "]"
fn parse_set_builder(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<ASTNode> {
    check_next_is!(it, Token::LBrack);

    let (set, ind) = get_or_err!(it, ind, parse_expression, "set builder");
    check_next_is!(it, Token::Ver);

    let mut conditions = Vec::new();
    match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::RBrack }) => (),
        _ => {
            let (condition, _) = get_or_err_direct!(it, ind, parse_expression, "tuple");
            conditions.push(condition);
        }
    }

    while let Some(t) = it.peek() {
        match *t {
            TokenPos { line: _, pos: _, token: Token::RBrack } => break,
            TokenPos { line: _, pos: _, token: Token::Comma } => {
                it.next();
                let (condition, _) = get_or_err_direct!(it, ind, parse_expression, "tuple");
                conditions.push(condition);
            }
            tp =>
                return Err(CustomErr { expected: "tuple element".to_string(), actual: tp.clone() })
        };
    }

    check_next_is!(it, Token::RBrack);
    return Ok((ASTNode::SetBuilder(set, conditions), ind));
}

// tuple ::= "(" [ ( maybe-expr { "," maybe-expr } ] ")"
pub fn parse_tuple(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<ASTNode> {
    check_next_is!(it, Token::LPar);

    let mut elements = Vec::new();
    match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::RPar }) => (),
        _ => {
            let (element, _) = get_or_err_direct!(it, ind, parse_expression, "tuple");
            elements.push(element);
        }
    }

    while let Some(t) = it.peek() {
        match *t {
            TokenPos { line: _, pos: _, token: Token::RPar } => break,
            TokenPos { line: _, pos: _, token: Token::Comma } => {
                it.next();
                let (expr, _) = get_or_err_direct!(it, ind, parse_expression, "tuple");
                elements.push(expr);
            }
            tp =>
                return Err(CustomErr { expected: "tuple element".to_string(), actual: tp.clone() })
        };
    }

    check_next_is!(it, Token::RPar);
    return Ok((ASTNode::FunTuple(elements), ind));
}

// "return" maybe-expression
fn parse_return(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<ASTNode> {
    check_next_is!(it, Token::Ret);
    if let Some(&&TokenPos { line: _, pos: _, token: Token::NL }) = it.peek() {
        return Ok((ASTNode::ReturnEmpty, ind));
    }

    let (expr, ind) = get_or_err!(it, ind, parse_expression, "return");
    return Ok((ASTNode::Return(expr), ind));
}
