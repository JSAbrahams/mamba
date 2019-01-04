use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::parse_expression;
use crate::parser::parse_do;
use crate::parser::parse_expression_or_do;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// expression ::= "(" ( expression-or-do | newline do ) ")" | ...
pub fn parse_bracket(it: &mut Peekable<Iter<Token>>, indent: i32)
                     -> (Result<ASTNode, String>, i32) {
    assert_eq!(it.next(), Some(&Token::LPar));
    if it.peek() == Some(&Token::NL) {
        it.next();
        let (expr, new_indent) = parse_do(it, indent);
    } else {
        let (expr, new_indent) = parse_expression_or_do(it, indent);
    }

    return match it.next() {
        Some(Token::RPar) => (expr, new_indent),

        Some(_) => (Err("Expecting closing bracket.".to_string()), new_indent),
        None => (Err("Expected closing bracket, but end of file.".to_string()), new_indent)
    };
}

// expression ::= ... | "return" expression | ...
pub fn parse_return(it: &mut Peekable<Iter<Token>>, indent: i32) -> (Result<ASTNode, String>, i32) {
    assert_eq!(it.next(), Some(&Token::Ret));
    return match parse_expression(it, indent) {
        (Ok(expr), new_indent) => (Ok(ASTNode::Return(Box::new(expr))), new_indent),
        err => err
    };
}
