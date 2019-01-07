use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::expression_or_statement::parse_maybe_expression;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

mod arithmetic;

// expression ::= "return" maybe-expression | arithmetic
pub fn parse(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::Ret) => parse_return(it, ind),
        Some(Token::Real(_)) | Some(Token::Int(_)) | Some(Token::ENum(_, _)) | Some(Token::Id(_)) |
        Some(Token::Str(_)) | Some(Token::Bool(_)) | Some(Token::Not) | Some(Token::Add) |
        Some(Token::Sub) => arithmetic::parse(it, ind),

        Some(t) => (Err(format!("Unexpected token while parsing expression: {:?}", t).to_string()),
                    ind),
        None => (Err("Unexpected end of file.".to_string()), ind)
    };
}

fn parse_return(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    debug_assert_eq!(it.next(), Some(&Token::Ret));

    return match parse_maybe_expression(it, ind) {
        (Ok(expr), new_ind) => (Ok(ASTNode::Return(wrap!(expr))), new_ind),
        err => err
    };
}
