use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::maybe_expr::parse_expression;
use crate::parser::parse_result::ParseResult;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// operation  ::= arithmetic | arithmetic relational maybe-expr
// arithmetic ::= term | unary arithmetic | term additive maybe-expr
// term       ::= factor | factor multiclative-operator maybe-expr
// factor     ::= constant | id

macro_rules! un_operator { ($it:expr, $ind:expr, $op:path) => {{
    $it.next(); match parse_expression($it, $ind) {
        (Ok(expr), ind) => (Ok($op(Box::new(expr))), ind),
        err => err
    }
}}}

macro_rules! bin_operator { ($factor:expr, $it:expr, $ind:expr, $op:path) => {{
    $it.next(); match parse_expression($it, $ind) {
        (Ok(expr), ind) => (Ok($op(Box::new($factor), Box::new(expr))), ind),
        err => err
    }
}}}

pub fn parse_operation(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    return match match it.peek() {
        Some(TokenPos::Id(_)) | Some(TokenPos::Real(_)) | Some(TokenPos::Int(_)) | Some(TokenPos::ENum(_, _)) |
        Some(TokenPos::Str(_)) | Some(TokenPos::Bool(_)) | Some(TokenPos::Not) | Some(TokenPos::Add) |
        Some(TokenPos::Sub) => parse_arithmetic(it, ind),
        Some(_) | None => (Err("Operation expected".to_string()), ind)
    } {
        (Ok(factor), ind) => match it.peek() {
            Some(TokenPos::Eq) => bin_operator!(factor, it, ind, ASTNode::Eq),
            Some(TokenPos::Is) => bin_operator!(factor, it, ind, ASTNode::Is),
            Some(TokenPos::IsN) => bin_operator!(factor, it, ind, ASTNode::IsN),
            Some(TokenPos::Neq) => bin_operator!(factor, it, ind, ASTNode::Neq),
            Some(TokenPos::Ge) => bin_operator!(factor, it, ind, ASTNode::Ge),
            Some(TokenPos::Geq) => bin_operator!(factor, it, ind, ASTNode::Geq),
            Some(TokenPos::Le) => bin_operator!(factor, it, ind, ASTNode::Le),
            Some(TokenPos::Leq) => bin_operator!(factor, it, ind, ASTNode::Leq),
            Some(TokenPos::And) => bin_operator!(factor, it, ind, ASTNode::And),
            Some(TokenPos::Or) => bin_operator!(factor, it, ind, ASTNode::Or),
            _ => (Ok(factor), ind)
        }
        err => err
    };
}

fn parse_arithmetic(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    return match match it.peek() {
        Some(TokenPos::Id(_)) | Some(TokenPos::Real(_)) | Some(TokenPos::Int(_)) | Some(TokenPos::ENum(_, _)) |
        Some(TokenPos::Str(_)) | Some(TokenPos::Bool(_)) => parse_term(it, ind),
        Some(TokenPos::Not) => un_operator!(it, ind, ASTNode::Not),
        Some(TokenPos::Add) => un_operator!(it, ind, ASTNode::AddU),
        Some(TokenPos::Sub) => un_operator!(it, ind, ASTNode::SubU),
        Some(_) | None => (Err("Expected arithmetic expression.".to_string()), ind)
    } {
        (Ok(term), ind) => match it.peek() {
            Some(TokenPos::Add) => bin_operator!(term, it, ind, ASTNode::Add),
            Some(TokenPos::Sub) => bin_operator!(term, it, ind, ASTNode::Sub),
            _ => (Ok(term), ind)
        }
        err => err
    };
}

fn parse_term(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    return match match it.peek() {
        Some(TokenPos::Id(_)) | Some(TokenPos::Str(_)) | Some(TokenPos::Real(_)) | Some(TokenPos::Int(_)) |
        Some(TokenPos::ENum(_, _)) | Some(TokenPos::Bool(_)) => parse_factor(it, ind),
        Some(_) | None => (Err("Expected term.".to_string()), ind)
    } {
        (Ok(factor), ind) => match it.peek() {
            Some(TokenPos::Mul) => bin_operator!(factor, it, ind, ASTNode::Mul),
            Some(TokenPos::Div) => bin_operator!(factor, it, ind, ASTNode::Div),
            Some(TokenPos::Pow) => bin_operator!(factor, it, ind, ASTNode::Pow),
            Some(TokenPos::Mod) => bin_operator!(factor, it, ind, ASTNode::Mod),
            _ => (Ok(factor), ind)
        }
        err => err
    };
}

fn parse_factor(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    return (match it.next() {
        Some(TokenPos::Id(id)) => Ok(ASTNode::Id(id.to_string())),
        Some(TokenPos::Str(string)) => Ok(ASTNode::Str(string.to_string())),
        Some(TokenPos::Real(real)) => Ok(ASTNode::Real(real.to_string())),
        Some(TokenPos::Int(int)) => Ok(ASTNode::Int(int.to_string())),
        Some(TokenPos::ENum(num, exp)) => Ok(ASTNode::ENum(num.to_string(), exp.to_string())),
        Some(TokenPos::Bool(boolean)) => Ok(ASTNode::Bool(*boolean)),
        Some(_) | None => Err("Expected factor.".to_string())
    }, ind);
}
