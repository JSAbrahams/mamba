use crate::common::*;
use mamba::lexer::tokenize;
use mamba::parser::parse;

#[test]
fn parse_assigns_and_while() {
    let source = resource_content(true, &["compound"], "assign_and_while.mamba");
    parse(&tokenize(&source).unwrap()).unwrap();
}
