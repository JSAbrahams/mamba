use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::maybe_expr::parse_expression;
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

pub fn parse_operation(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    return match match it.peek() {
        Some(Token::Id(_)) | Some(Token::Real(_)) | Some(Token::Int(_)) | Some(Token::ENum(_, _)) |
        Some(Token::Str(_)) | Some(Token::Bool(_)) | Some(Token::Not) | Some(Token::Add) |
        Some(Token::Sub) => parse_arithmetic(it, ind),
        Some(_) | None => (Err("Operation expected".to_string()), ind)
    } {
        (Ok(factor), ind) => match it.peek() {
            Some(Token::Eq) => bin_operator!(factor, it, ind, ASTNode::Eq),
            Some(Token::Is) => bin_operator!(factor, it, ind, ASTNode::Is),
            Some(Token::IsN) => bin_operator!(factor, it, ind, ASTNode::IsN),
            Some(Token::Neq) => bin_operator!(factor, it, ind, ASTNode::Neq),
            Some(Token::Ge) => bin_operator!(factor, it, ind, ASTNode::Ge),
            Some(Token::Geq) => bin_operator!(factor, it, ind, ASTNode::Geq),
            Some(Token::Le) => bin_operator!(factor, it, ind, ASTNode::Le),
            Some(Token::Leq) => bin_operator!(factor, it, ind, ASTNode::Leq),
            Some(Token::And) => bin_operator!(factor, it, ind, ASTNode::And),
            Some(Token::Or) => bin_operator!(factor, it, ind, ASTNode::Or),
            _ => (Ok(factor), ind)
        }
        err => err
    };
}

fn parse_arithmetic(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    return match match it.peek() {
        Some(Token::Id(_)) | Some(Token::Real(_)) | Some(Token::Int(_)) | Some(Token::ENum(_, _)) |
        Some(Token::Str(_)) | Some(Token::Bool(_)) => parse_term(it, ind),
        Some(Token::Not) => un_operator!(it, ind, ASTNode::Not),
        Some(Token::Add) => un_operator!(it, ind, ASTNode::AddU),
        Some(Token::Sub) => un_operator!(it, ind, ASTNode::SubU),
        Some(_) | None => panic!("Expected arithmetic expression.")
    } {
        (Ok(term), ind) => match it.peek() {
            Some(Token::Add) => bin_operator!(term, it, ind, ASTNode::Add),
            Some(Token::Sub) => bin_operator!(term, it, ind, ASTNode::Sub),
            _ => (Ok(term), ind)
        }
        err => err
    };
}

fn parse_term(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    return match match it.peek() {
        Some(Token::Id(_)) | Some(Token::Str(_)) | Some(Token::Real(_)) | Some(Token::Int(_)) |
        Some(Token::ENum(_, _)) | Some(Token::Bool(_)) => parse_factor(it, ind),
        Some(_) | None => panic!("Expected term.")
    } {
        (Ok(factor), ind) => match it.peek() {
            Some(Token::Mul) => bin_operator!(factor, it, ind, ASTNode::Mul),
            Some(Token::Div) => bin_operator!(factor, it, ind, ASTNode::Div),
            Some(Token::Pow) => bin_operator!(factor, it, ind, ASTNode::Pow),
            Some(Token::Mod) => bin_operator!(factor, it, ind, ASTNode::Mod),
            _ => (Ok(factor), ind)
        }
        err => err
    };
}

fn parse_factor(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    return (match it.next() {
        Some(Token::Id(id)) => Ok(ASTNode::Id(id.to_string())),
        Some(Token::Str(string)) => Ok(ASTNode::Str(string.to_string())),
        Some(Token::Real(real)) => Ok(ASTNode::Real(real.to_string())),
        Some(Token::Int(int)) => Ok(ASTNode::Int(int.to_string())),
        Some(Token::ENum(num, exp)) => Ok(ASTNode::ENum(num.to_string(), exp.to_string())),
        Some(Token::Bool(boolean)) => Ok(ASTNode::Bool(*boolean)),
        Some(_) | None => panic!("Expected factor.")
    }, ind);
}
