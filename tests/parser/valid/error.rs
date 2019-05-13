use crate::common::valid_resource_content;
use mamba::lexer::tokenize;
use mamba::parser::parse;

#[test]
fn handle_verify() {
    let source = valid_resource_content(&["error"], "handle.mamba");
    assert!(parse(&tokenize(&source).unwrap()).is_ok());
}

#[test]
fn raises_verify() {
    let source = valid_resource_content(&["error"], "raise.mamba");
    assert!(parse(&tokenize(&source).unwrap()).is_ok());
}
