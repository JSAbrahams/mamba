use crate::common::*;
use mamba::core::to_py_source;
use mamba::desugar::desugar;
use mamba::lexer::tokenize;
use mamba::parser::parse;

#[test]
fn core_list() {
    let source = valid_resource_content(&["collection"], "tuple.mamba");
    to_py!(source);
}

#[test]
#[ignore]
fn core_map() {
    let source = valid_resource_content(&["collection"], "map.mamba");
    to_py!(source);
}

#[test]
fn core_set() {
    let source = valid_resource_content(&["collection"], "set.mamba");
    to_py!(source);
}

#[test]
fn core_tuple() {
    let source = valid_resource_content(&["collection"], "tuple.mamba");
    to_py!(source);
}
