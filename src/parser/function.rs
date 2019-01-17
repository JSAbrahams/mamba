use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::maybe_expr::parse_tuple;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::TPIterator;
use std::env;

pub fn parse_function_call(caller: ASTNode, it: &mut TPIterator) -> ParseResult {
    print_parse!(it, "function call");
    check_next_is!(it, Token::Point);

    panic!("not implemented")
}

pub fn parse_function_call_direct(name: ASTNode, it: &mut TPIterator) -> ParseResult {
    print_parse!(it, "function call direct");

    panic!("not implemented")
}

pub fn parse_function_definition_body(it: &mut TPIterator) -> ParseResult {
    print_parse!(it, "function definition");
    check_next_is!(it, Token::Fun);

    panic!("not implemented")
}

pub fn parse_function_anonymous(it: &mut TPIterator) -> ParseResult {
    panic!("not implemented")
}
