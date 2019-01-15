use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::maybe_expr::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;
use std::env;

macro_rules! u_op { ($it:expr, $ind:expr, $fun:path, $op:path) => {{
    $it.next(); match $fun($it, $ind) {
        Ok((expr, ind)) => Ok(($op(Box::new(expr)), ind)),
        err => err
    }
}}}

macro_rules! b_op { ($left:expr, $it:expr, $ind:expr, $fun:path, $op:path) => {{
    $it.next(); match $fun($it, $ind) {
        Ok((expr, ind)) => Ok(($op(Box::new($left), Box::new(expr)), ind)),
        err => err
    }
}}}

pub fn parse_operation(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<ASTNode> {
    print_parse!(it, ind, "operation");
    let (relation, ind) = get_or_err_direct!(it, ind, parse_relation, "operation");

    return match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::Eq }) =>
            b_op!(relation, it, ind, parse_operation, ASTNode::Eq),
        Some(TokenPos { line: _, pos: _, token: Token::Is }) =>
            b_op!(relation, it, ind, parse_operation, ASTNode::Is),
        Some(TokenPos { line: _, pos: _, token: Token::IsN }) =>
            b_op!(relation, it, ind, parse_operation, ASTNode::IsN),
        Some(TokenPos { line: _, pos: _, token: Token::Neq }) =>
            b_op!(relation, it, ind, parse_operation, ASTNode::Neq),
        Some(TokenPos { line: _, pos: _, token: Token::And }) =>
            b_op!(relation, it, ind, parse_operation, ASTNode::And),
        Some(TokenPos { line: _, pos: _, token: Token::Or }) =>
            b_op!(relation, it, ind, parse_operation, ASTNode::Or),
        Some(TokenPos { line: _, pos: _, token: Token::IsA }) =>
            b_op!(relation, it, ind, parse_operation, ASTNode::IsA),
        _ => Ok((relation, ind))
    };
}

fn parse_relation(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<ASTNode> {
    let (arithmetic, ind) = get_or_err_direct!(it, ind, parse_arithmetic, "comparison");

    return match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::Ge }) =>
            b_op!(arithmetic, it, ind, parse_relation, ASTNode::Ge),
        Some(TokenPos { line: _, pos: _, token: Token::Geq }) =>
            b_op!(arithmetic, it, ind, parse_relation, ASTNode::Geq),
        Some(TokenPos { line: _, pos: _, token: Token::Le }) =>
            b_op!(arithmetic, it, ind, parse_relation, ASTNode::Le),
        Some(TokenPos { line: _, pos: _, token: Token::Leq }) =>
            b_op!(arithmetic, it, ind, parse_relation, ASTNode::Leq),
        _ => Ok((arithmetic, ind))
    };
}

fn parse_arithmetic(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<ASTNode> {
    let (term, ind) = get_or_err_direct!(it, ind, parse_term, "arithmetic");

    match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::Add }) =>
            b_op!(term, it, ind, parse_arithmetic, ASTNode::Add),
        Some(TokenPos { line: _, pos: _, token: Token::Sub }) =>
            b_op!(term, it, ind, parse_arithmetic, ASTNode::Sub),
        _ => Ok((term, ind))
    }
}

fn parse_term(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<ASTNode> {
    let (factor, ind) = get_or_err_direct!(it, ind, parse_inner_term, "term");

    return match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::Mul }) =>
            b_op!(factor, it, ind, parse_term, ASTNode::Mul),
        Some(TokenPos { line: _, pos: _, token: Token::Div }) =>
            b_op!(factor, it, ind, parse_term, ASTNode::Div),
        _ => Ok((factor, ind))
    };
}

fn parse_inner_term(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<ASTNode> {
    let (factor, ind) = get_or_err_direct!(it, ind, parse_factor, "inner term");

    return match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::Pow }) =>
            b_op!(factor, it, ind, parse_inner_term, ASTNode::Pow),
        Some(TokenPos { line: _, pos: _, token: Token::Mod }) =>
            b_op!(factor, it, ind, parse_inner_term, ASTNode::Mod),
        _ => Ok((factor, ind))
    };
}

fn parse_factor(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<ASTNode> {
    return match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::Not }) =>
            u_op!(it, ind, parse_factor, ASTNode::Not),
        Some(TokenPos { line: _, pos: _, token: Token::Add }) =>
            u_op!(it, ind, parse_factor, ASTNode::AddU),
        Some(TokenPos { line: _, pos: _, token: Token::Sub }) =>
            u_op!(it, ind, parse_factor, ASTNode::SubU),
        Some(TokenPos { line: _, pos: _, token: Token::Sqrt }) =>
            u_op!(it, ind, parse_factor, ASTNode::Sqrt),

        _ => {
            return match it.next() {
                Some(TokenPos { line: _, pos: _, token: Token::Id(id) }) =>
                    Ok((ASTNode::Id(id.to_string()), 0)),
                Some(TokenPos { line: _, pos: _, token: Token::Str(string) }) =>
                    Ok((ASTNode::Str(string.to_string()), 0)),
                Some(TokenPos { line: _, pos: _, token: Token::Real(real) }) =>
                    Ok((ASTNode::Real(real.to_string()), 0)),
                Some(TokenPos { line: _, pos: _, token: Token::Int(int) }) =>
                    Ok((ASTNode::Int(int.to_string()), 0)),
                Some(TokenPos { line: _, pos: _, token: Token::ENum(num, exp) }) =>
                    Ok((ASTNode::ENum(num.to_string(), exp.to_string()), 0)),
                Some(TokenPos { line: _, pos: _, token: Token::Bool(boolean) }) =>
                    Ok((ASTNode::Bool(*boolean), 0)),
                Some(_) => parse_expression(it, ind),
                None => Err(CustomEOFErr { expected: "factor".to_string() })
            }
        }
    };
}
