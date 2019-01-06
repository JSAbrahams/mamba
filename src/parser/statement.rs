use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::parse_expression;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

//statement ::= "print" expression | ...
pub fn parse_print(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    debug_assert_eq!(it.next(), Some(&Token::Print));

    match parse_expression(it, ind) {
        (Ok(expr), new_indent) => (Ok(ASTNode::Print(Box::new(expr))), new_indent),
        err => err
    }
}
