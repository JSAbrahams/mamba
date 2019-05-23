use crate::common::*;
use mamba::core::to_py_source;
use mamba::desugar::desugar;
use mamba::lexer::tokenize;
use mamba::parser::parse;

#[test]
fn core_assigns_and_while() {
    let source = resource_content(true, &["compound"], "assign_and_while.mamba");
    to_py!(source);
}
