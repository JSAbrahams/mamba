use crate::util::*;
use mamba::lexer::tokenize;
use mamba::parser::parse;

#[test]
fn parse_assigns_and_while() {
    let source = valid_resource_content(&["compound"], "assign_and_while.mamba");
    parse(&tokenize(&source).unwrap()).unwrap();
}
