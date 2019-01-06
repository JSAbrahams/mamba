use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::parse_do;
use crate::parser::parse_expression;
use crate::parser::parse_expression_or_do;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// expression ::= "(" ( expression-or-do | newline do ) ")" | ...
pub fn parse_bracket(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    debug_assert_eq!(it.next(), Some(&Token::LPar));

    let (expr_or_do, new_ind) = parse_expression_or_do(it, ind);
    return match it.next() {
        Some(Token::RPar) => (expr_or_do, new_ind),

        Some(_) => (Err("Expecting closing bracket.".to_string()), new_ind),
        None => (Err("Expected closing bracket, but end of file.".to_string()), new_ind)
    };
}

// expression ::= ... | "return" expression | ...
pub fn parse_return(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    debug_assert_eq!(it.next(), Some(&Token::Ret));

    return match parse_expression(it, ind) {
        (Ok(expr), new_ind) => (Ok(ASTNode::Return(Box::new(expr))), new_ind),
        err => err
    };
}
