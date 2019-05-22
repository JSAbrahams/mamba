use crate::common::resource_content;
use mamba::lexer::tokenize;
use mamba::parser::parse;

#[test]
fn handle_verify() {
    let source = resource_content(true, &["error"], "handle.mamba");
    assert!(parse(&tokenize(&source).unwrap()).is_ok());
}

#[test]
fn raises_verify() {
    let source = resource_content(true, &["error"], "raise.mamba");
    assert!(parse(&tokenize(&source).unwrap()).is_ok());
}

#[test]
fn with_verify() {
    let source = valid_resource_content(&["error"], "with.mamba");
    assert!(parse(&tokenize(&source).unwrap()).is_ok());
}
