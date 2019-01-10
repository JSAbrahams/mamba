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
        (Ok(expr), ind) => (Ok($op(Box::new(expr))), ind),
        err => err
    }
}}}

macro_rules! b_op { ($factor:expr, $it:expr, $ind:expr, $op:path) => {{
    $it.next(); match parse_expression($it, $ind) {
        (Ok(expr), ind) => (Ok($op(Box::new($factor), Box::new(expr))), ind),
        err => err
    }
}}}

pub fn parse_operation(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    return match match it.peek() {
        Some(TokenPos { line, pos, token: Token::Id(_) }) |
        Some(TokenPos { line, pos, token: Token::Real(_) }) |
        Some(TokenPos { line, pos, token: Token::Int(_) }) |
        Some(TokenPos { line, pos, token: Token::ENum(_, _) }) |
        Some(TokenPos { line, pos, token: Token::Str(_) }) |
        Some(TokenPos { line, pos, token: Token::Bool(_) }) |
        Some(TokenPos { line, pos, token: Token::Not }) |
        Some(TokenPos { line, pos, token: Token::Add }) |
        Some(TokenPos { line, pos, token: Token::Sub }) => parse_arithmetic(it, ind),

        Some(actual) => (TokenErr { expected: Token::Add, actual }, ind),
        None => (EOFErr { expected: Token::Add }, ind)
    } {
        (Ok(factor), ind) => match it.peek() {
            Some(TokenPos { line, pos, token: Token::Eq }) => b_op!(factor, it, ind, ASTNode::Eq),
            Some(TokenPos { line, pos, token: Token::Is }) => b_op!(factor, it, ind, ASTNode::Is),
            Some(TokenPos { line, pos, token: Token::IsN }) => b_op!(factor, it, ind, ASTNode::IsN),
            Some(TokenPos { line, pos, token: Token::Neq }) => b_op!(factor, it, ind, ASTNode::Neq),
            Some(TokenPos { line, pos, token: Token::Ge }) => b_op!(factor, it, ind, ASTNode::Ge),
            Some(TokenPos { line, pos, token: Token::Geq }) => b_op!(factor, it, ind, ASTNode::Geq),
            Some(TokenPos { line, pos, token: Token::Le }) => b_op!(factor, it, ind, ASTNode::Le),
            Some(TokenPos { line, pos, token: Token::Leq }) => b_op!(factor, it, ind, ASTNode::Leq),
            Some(TokenPos { line, pos, token: Token::And }) => b_op!(factor, it, ind, ASTNode::And),
            Some(TokenPos { line, pos, token: Token::Or }) => b_op!(factor, it, ind, ASTNode::Or),
            _ => (Ok(factor), ind)
        }
        err => err
    };
}

fn parse_arithmetic(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    return match match it.peek() {
        Some(TokenPos { line, pos, token: Token::Id(_) }) |
        Some(TokenPos { line, pos, token: Token::Real(_) }) |
        Some(TokenPos { line, pos, token: Token::Int(_) }) |
        Some(TokenPos { line, pos, token: Token::ENum(_, _) }) |
        Some(TokenPos { line, pos, token: Token::Str(_) }) |
        Some(TokenPos { line, pos, token: Token::Bool(_) }) => parse_term(it, ind),

        Some(TokenPos { line, pos, token: Token::Not }) => u_op!(it, ind, ASTNode::Not),
        Some(TokenPos { line, pos, token: Token::Add }) => u_op!(it, ind, ASTNode::AddU),
        Some(TokenPos { line, pos, token: Token::Sub }) => u_op!(it, ind, ASTNode::SubU),

        Some(actual) => (TokenErr { expected: Token::Add, actual }, ind),
        None => (EOFErr { expected: Token::Add }, ind)
    } {
        (Ok(term), ind) => match it.peek() {
            Some(TokenPos { line, pos, token: Token::Add }) => b_op!(term, it, ind, ASTNode::Add),
            Some(TokenPos { line, pos, token: Token::Sub }) => b_op!(term, it, ind, ASTNode::Sub),
            _ => (Ok(term), ind)
        }
        err => err
    };
}

fn parse_term(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    return match match it.peek() {
        Some(TokenPos { line, pos, token: Token::Id(_) }) |
        Some(TokenPos { line, pos, token: Token::Str(_) }) |
        Some(TokenPos { line, pos, token: Token::Real(_) }) |
        Some(TokenPos { line, pos, token: Token::Int(_) }) |
        Some(TokenPos { line, pos, token: Token::ENum(_, _) }) |
        Some(TokenPos { line, pos, token: Token::Bool(_) }) => parse_factor(it, ind),

        Some(actual) => (TokenErr { expected: Token::Add, actual }, ind),
        None => (EOFErr { expected: Token::Add }, ind)
    } {
        (Ok(factor), ind) => match it.peek() {
            Some(TokenPos { line, pos, token: Token::Mul }) => b_op!(factor, it, ind, ASTNode::Mul),
            Some(TokenPos { line, pos, token: Token::Div }) => b_op!(factor, it, ind, ASTNode::Div),
            Some(TokenPos { line, pos, token: Token::Pow }) => b_op!(factor, it, ind, ASTNode::Pow),
            Some(TokenPos { line, pos, token: Token::Mod }) => b_op!(factor, it, ind, ASTNode::Mod),
            _ => (Ok(factor), ind)
        }
        err => err
    };
}

fn parse_factor(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    return (match it.next() {
        Some(TokenPos { line, pos, token: Token::Id(id) }) => Ok(ASTNode::Id(id.to_string())),
        Some(TokenPos { line, pos, token: Token::Str(string) }) =>
            Ok(ASTNode::Str(string.to_string())),
        Some(TokenPos { line, pos, token: Token::Real(real) }) =>
            Ok(ASTNode::Real(real.to_string())),
        Some(TokenPos { line, pos, token: Token::Int(int) }) => Ok(ASTNode::Int(int.to_string())),
        Some(TokenPos { line, pos, token: Token::ENum(num, exp) }) =>
            Ok(ASTNode::ENum(num.to_string(), exp.to_string())),
        Some(TokenPos { line, pos, token: Token::Bool(boolean) }) => Ok(ASTNode::Bool(*boolean)),

        Some(actual) => Err(TokenErr { expected: Token::Add, actual }),
        None => Err(EOFErr { expected: Token::Add })
    }, ind);
}
