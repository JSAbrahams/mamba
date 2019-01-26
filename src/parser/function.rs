use crate::lexer::token::Token;
use crate::parser::ASTNodePos;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::TPIterator;

pub fn parse_function_call(caller: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    panic!("not implemented")
}

pub fn parse_function_call_direct(name: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    panic!("not implemented")
}

pub fn parse_function_definition_body(it: &mut TPIterator) -> ParseResult {
    panic!("not implemented")
}

pub fn parse_function_anonymous(it: &mut TPIterator) -> ParseResult {
    panic!("not implemented")
}
