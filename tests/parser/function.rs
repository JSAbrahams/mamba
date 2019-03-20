use crate::util::*;
use mamba::lexer::tokenize;
use mamba::parser::parse;

#[test]
fn parse_function_definitions() {
    let source = valid_resource_content(&["function"], "definition.mamba");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_function_calling() {
    let source = valid_resource_content(&["function"], "calls.mamba");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_infix_function_calling() {
    let source = valid_resource_content(&["function"], "infix_calls.mamba");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}
