use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::expression::parse as parse_expression;
use crate::parser::parse_expression_or_do;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

mod arithmetic;
mod control_flow;

// expression ::= "(" expression-or-do ")" | "return" expression | arithmetic | control-flow-expr
pub fn parse(it: &mut Peekable<Iter<Token>>, ind: i32)
             -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::LPar) => parse_bracket(it, ind),
        Some(Token::Ret) => parse_return(it, ind),
        Some(Token::Real(_)) | Some(Token::Int(_)) | Some(Token::ENum(_, _)) | Some(Token::Id(_)) |
        Some(Token::Str(_)) | Some(Token::Bool(_)) | Some(Token::Not) | Some(Token::Add) |
        Some(Token::Sub) => arithmetic::parse(it, ind),
        Some(Token::If) | Some(Token::Unless) | Some(Token::When) => control_flow::parse(it, ind),

        Some(t) => (Err(format!("Unexpected token while parsing expression: {:?}", t).to_string()),
                    ind),
        None => (Err("Unexpected end of file.".to_string()), ind)
    };
}

// expression ::= "(" ( expression-or-do | newline do ) ")" | ...
fn parse_bracket(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    debug_assert_eq!(it.next(), Some(&Token::LPar));

    let (expr_or_do, new_ind) = parse_expression_or_do(it, ind);
    return match it.next() {
        Some(Token::RPar) => (expr_or_do, new_ind),

        Some(_) => (Err("Expecting closing bracket.".to_string()), new_ind),
        None => (Err("Expected closing bracket, but end of file.".to_string()), new_ind)
    };
}

// expression ::= ... | "return" expression | ...
fn parse_return(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    debug_assert_eq!(it.next(), Some(&Token::Ret));

    return match parse_expression(it, ind) {
        (Ok(expr), new_ind) => (Ok(ASTNode::Return(Box::new(expr))), new_ind),
        err => err
    };
}
