use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::parse_expression;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

macro_rules! un_operator { ($it:expr, $ind:expr, $op:path) => {{
    $it.next();
    match parse_expression($it, $ind) {
        (Ok(expr), new_ind) => (Ok($op(Box::new(expr))), new_ind),
        err => err
    }
}}}

macro_rules! bin_operator { ($factor:expr, $it:expr, $ind:expr, $op:path) => {{
    $it.next();
    match parse_expression($it, $ind) {
        (Ok(expr), new_ind) => (Ok($op(Box::new($factor), Box::new(expr))), new_ind),
        err => err
    }
}}}

// arithmetic-expression ::= term | unary-operator expression | term additive-operator expression
pub fn parse(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::Id(_)) | Some(Token::Real(_)) | Some(Token::Int(_)) | Some(Token::ENum(_, _)) |
        Some(Token::Str(_)) | Some(Token::Bool(_)) => match parse_term(it, ind) {
            (Ok(term), new_ind) => match it.peek() {
                Some(Token::Add) => bin_operator!(term, it, new_ind, ASTNode::Add),
                Some(Token::Sub) => bin_operator!(term, it, new_ind, ASTNode::Sub),
                _ => (Ok(term), new_ind)
            }
            err => err
        }
        Some(Token::Not) => un_operator!(it, ind, ASTNode::Not),
        Some(Token::Add) => un_operator!(it, ind, ASTNode::AddU),
        Some(Token::Sub) => un_operator!(it, ind, ASTNode::SubU),

        Some(_) => panic!("Expected arithmetic expression, but other token."),
        None => panic!("Expected arithmetic expression, but end of file.")
    };
}

// term ::= factor | factor multiclative-operator expression
fn parse_term(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::Id(_)) | Some(Token::Str(_)) | Some(Token::Real(_)) | Some(Token::Int(_)) |
        Some(Token::ENum(_, _)) | Some(Token::Bool(_)) =>
            match parse_factor(it, ind) {
                (Ok(factor), new_ind) => match it.peek() {
                    Some(Token::Mul) => bin_operator!(factor, it, new_ind, ASTNode::Mul),
                    Some(Token::Div) => bin_operator!(factor, it, new_ind, ASTNode::Div),
                    Some(Token::Pow) => bin_operator!(factor, it, new_ind, ASTNode::Pow),
                    Some(Token::Mod) => bin_operator!(factor, it, new_ind, ASTNode::Mod),
                    Some(Token::Eq) => bin_operator!(factor, it, new_ind, ASTNode::Eq),
                    Some(Token::Is) => bin_operator!(factor, it, new_ind, ASTNode::Is),
                    Some(Token::IsN) => bin_operator!(factor, it, new_ind, ASTNode::IsN),
                    Some(Token::Neq) => bin_operator!(factor, it, new_ind, ASTNode::Neq),
                    Some(Token::Ge) => bin_operator!(factor, it, new_ind, ASTNode::Ge),
                    Some(Token::Geq) => bin_operator!(factor, it, new_ind, ASTNode::Geq),
                    Some(Token::Le) => bin_operator!(factor, it, new_ind, ASTNode::Le),
                    Some(Token::Leq) => bin_operator!(factor, it, new_ind, ASTNode::Leq),
                    Some(Token::And) => bin_operator!(factor, it, new_ind, ASTNode::And),
                    Some(Token::Or) => bin_operator!(factor, it, new_ind, ASTNode::Or),
                    _ => (Ok(factor), new_ind)
                }
                err => err
            },

        Some(_) => panic!("Expected term, but other."),
        None => panic!("Expected term, but end of file.")
    };
}

// factor ::= constant | id
fn parse_factor(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    return (match it.next() {
        Some(Token::Id(id)) => Ok(ASTNode::Id(id.to_string())),
        Some(Token::Str(string)) => Ok(ASTNode::Str(string.to_string())),
        Some(Token::Real(real)) => Ok(ASTNode::Real(real.to_string())),
        Some(Token::Int(int)) => Ok(ASTNode::Int(int.to_string())),
        Some(Token::ENum(num, exp)) => Ok(ASTNode::ENum(num.to_string(), exp.to_string())),
        Some(Token::Bool(boolean)) => Ok(ASTNode::Bool(*boolean)),

        Some(_) => panic!("Expected factor, but other."),
        None => panic!("Expected factor, but end of file.")
    }, ind);
}
