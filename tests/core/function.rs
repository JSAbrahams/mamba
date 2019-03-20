use crate::util::*;
use mamba::core::to_py_source;
use mamba::desugarer::desugar;
use mamba::lexer::tokenize;
use mamba::parser::parse;

#[test]
fn core_function_definitions() {
    let source = valid_resource_content(&["function"], "definition.mamba");
    to_py!(source);
}

#[test]
fn core_function_calling() {
    let source = valid_resource_content(&["function"], "calls.mamba");
    to_py!(source);
}

#[test]
fn core_infix_function_calling() {
    let source = valid_resource_content(&["function"], "infix_calls.mamba");
    to_py!(source);
}
