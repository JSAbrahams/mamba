use crate::common::resource_content;
use mamba::lexer::tokenize;
use mamba::parser::parse;

#[test]
fn assign_and_while() {
    let source = resource_content(false, &[], "assign_and_while.mamba");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}
