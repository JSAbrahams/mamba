use crate::common::valid_resource_content;
use mamba::lexer::tokenize;
use mamba::parser::parse;

#[test]
fn handle_verify() {
    let source = valid_resource_content(&["error"], "handle.mamba");
    parse(&tokenize(&source).unwrap());
}

#[test]
fn raises_verify() {
    let source = valid_resource_content(&["error"], "raise.mamba");
    parse(&tokenize(&source).unwrap());
}
