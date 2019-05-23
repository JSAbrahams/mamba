use crate::common::*;
use mamba::lexer::tokenize;
use mamba::parser::parse;

#[test]
#[ignore]
fn function_definitions() {
    let source = resource_content(true, &["function"], "definition.mamba");
    parse(&tokenize(&source).unwrap()).unwrap();
}

#[test]
fn function_calling() {
    let source = resource_content(true, &["function"], "calls.mamba");
    parse(&tokenize(&source).unwrap()).unwrap();
}

#[test]
fn infix_function_calling() {
    let source = resource_content(true, &["function"], "infix_calls.mamba");
    parse(&tokenize(&source).unwrap()).unwrap();
}
