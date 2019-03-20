use crate::util::*;
use mamba::core::to_py_source;
use mamba::desugarer::desugar;
use mamba::lexer::tokenize;
use mamba::parser::parse;

#[test]
fn core_assigns_and_while() {
    let source = valid_resource_content(&["compound"], "assign_and_while.mamba");
    to_py!(source);
}
