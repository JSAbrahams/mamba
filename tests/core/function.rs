use crate::common::*;
use mamba::core::to_source;
use mamba::desugar::desugar;
use mamba::lex::tokenize;
use mamba::parse::parse;

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
