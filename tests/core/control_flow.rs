use crate::util::*;
use mamba::core::to_py_source;
use mamba::desugar::desugar;
use mamba::lexer::tokenize;
use mamba::parser::parse;

#[test]
fn core_for_statements() {
    let source = valid_resource_content(&["control_flow"], "for_statements.mamba");
    to_py!(source);
}

#[test]
fn core_if() {
    let source = valid_resource_content(&["control_flow"], "if.mamba");
    to_py!(source);
}

#[test]
#[ignore]
fn core_match_statements() {
    let source = valid_resource_content(&["control_flow"], "match_statements.mamba");
    to_py!(source);
}

#[test]
fn core_while_statements() {
    let source = valid_resource_content(&["control_flow"], "while_statements.mamba");
    to_py!(source);
}
