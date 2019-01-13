use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::maybe_expr::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// operation  ::= arithmetic | arithmetic relational maybe-expr
// arithmetic ::= term | unary arithmetic | term additive maybe-expr
// term       ::= factor | factor multiclative-operator maybe-expr
// factor     ::= constant | id

macro_rules! u_op { ($it:expr, $ind:expr, $op:path) => {{
    $it.next(); match parse_expression($it, $ind) {
        Ok((expr, ind)) => Ok(($op(Box::new(expr)), ind)),
        err => err
    }
}}}

macro_rules! b_op { ($factor:expr, $it:expr, $ind:expr, $op:path) => {{
    $it.next(); match parse_expression($it, $ind) {
        Ok((expr, ind)) => Ok(($op(Box::new($factor), Box::new(expr)), ind)),
        err => err
    }
}}}

pub fn parse_operation(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<ASTNode> {
    let (arithmetic, ind) = get_or_err_direct!(it, ind, parse_arithmetic, "operation");

    return match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::Eq }) =>
            b_op!(arithmetic, it, ind, ASTNode::Eq),
        Some(TokenPos { line: _, pos: _, token: Token::Is }) =>
            b_op!(arithmetic, it, ind, ASTNode::Is),
        Some(TokenPos { line: _, pos: _, token: Token::IsN }) =>
            b_op!(arithmetic, it, ind, ASTNode::IsN),
        Some(TokenPos { line: _, pos: _, token: Token::Neq }) =>
            b_op!(arithmetic, it, ind, ASTNode::Neq),
        Some(TokenPos { line: _, pos: _, token: Token::Ge }) =>
            b_op!(arithmetic, it, ind, ASTNode::Ge),
        Some(TokenPos { line: _, pos: _, token: Token::Geq }) =>
            b_op!(arithmetic, it, ind, ASTNode::Geq),
        Some(TokenPos { line: _, pos: _, token: Token::Le }) =>
            b_op!(arithmetic, it, ind, ASTNode::Le),
        Some(TokenPos { line: _, pos: _, token: Token::Leq }) =>
            b_op!(arithmetic, it, ind, ASTNode::Leq),
        Some(TokenPos { line: _, pos: _, token: Token::And }) =>
            b_op!(arithmetic, it, ind, ASTNode::And),
        Some(TokenPos { line: _, pos: _, token: Token::Or }) =>
            b_op!(arithmetic, it, ind, ASTNode::Or),
        _ => Ok((arithmetic, ind))
    };
}

fn parse_arithmetic(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<ASTNode> {
    return match match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::Not }) =>
            u_op!(it, ind, ASTNode::Not),
        Some(TokenPos { line: _, pos: _, token: Token::Add }) =>
            u_op!(it, ind, ASTNode::AddU),
        Some(TokenPos { line: _, pos: _, token: Token::Sub }) =>
            u_op!(it, ind, ASTNode::SubU),

        _ => parse_term(it, ind)
    } {
        Ok((term, ind)) => match it.peek() {
            Some(TokenPos { line: _, pos: _, token: Token::Add }) =>
                b_op!(term, it, ind, ASTNode::Add),
            Some(TokenPos { line: _, pos: _, token: Token::Sub }) =>
                b_op!(term, it, ind, ASTNode::Sub),
            _ => Ok((term, ind))
        }
        err => err
    };
}

fn parse_term(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<ASTNode> {
    let (factor, ind) = get_or_err_direct!(it, ind, parse_factor, "term");

    return match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::Mul }) =>
            b_op!(factor, it, ind, ASTNode::Mul),
        Some(TokenPos { line: _, pos: _, token: Token::Div }) =>
            b_op!(factor, it, ind, ASTNode::Div),
        Some(TokenPos { line: _, pos: _, token: Token::Pow }) =>
            b_op!(factor, it, ind, ASTNode::Pow),
        Some(TokenPos { line: _, pos: _, token: Token::Mod }) =>
            b_op!(factor, it, ind, ASTNode::Mod),
        _ => Ok((factor, ind))
    };
}

fn parse_factor(it: &mut Peekable<Iter<TokenPos>>, _ind: i32) -> ParseResult<ASTNode> {
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

        Some(next) => Err(CustomErr { expected: "factor".to_string(), actual: next.clone() }),
        None => Err(CustomEOFErr { expected: "factor".to_string() })
    };
}
