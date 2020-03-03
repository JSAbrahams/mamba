use crate::common::*;
use mamba::lex::tokenize;
use mamba::parse::parse;

#[test]
fn function_definitions() {
    let source = resource_content(true, &["function"], "definition.mamba");
    parse(&tokenize(&source).unwrap()).unwrap();
}

#[test]
fn function_calling() {
    let source = resource_content(true, &["function"], "calls.mamba");
    parse(&tokenize(&source).unwrap()).unwrap();
}
