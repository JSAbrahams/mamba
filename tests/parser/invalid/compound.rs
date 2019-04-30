use crate::common::*;
use mamba::lexer::tokenize;
use mamba::parser::parse;

#[test]
fn assigns_and_while() {
    let source = invalid_resource_content(&["syntax"], "assign_and_while.mamba");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}
