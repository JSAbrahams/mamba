use mamba::lex::tokenize;
use mamba::parse::parse;

use crate::common::*;

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
