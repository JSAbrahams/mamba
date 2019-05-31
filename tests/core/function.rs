use crate::common::*;
use mamba::core::to_source;
use mamba::desugar::desugar;
use mamba::lexer::tokenize;
use mamba::parser::parse;

#[test]
fn core_function_definitions() {
    let source = resource_content(true, &["function"], "definition.mamba");
    to_py!(source);
}

#[test]
fn core_function_calling() {
    let source = resource_content(true, &["function"], "calls.mamba");
    to_py!(source);
}

#[test]
fn core_infix_function_calling() {
    let source = resource_content(true, &["function"], "infix_calls.mamba");
    to_py!(source);
}
