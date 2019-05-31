use crate::common::*;
use mamba::core::to_source;
use mamba::desugar::desugar;
use mamba::lexer::tokenize;
use mamba::parser::parse;

#[test]
fn core_list() {
    let source = resource_content(true, &["collection"], "tuple.mamba");
    to_py!(source);
}

#[test]
#[ignore]
fn core_map() {
    let source = resource_content(true, &["collection"], "map.mamba");
    to_py!(source);
}

#[test]
fn core_set() {
    let source = resource_content(true, &["collection"], "set.mamba");
    to_py!(source);
}

#[test]
fn core_tuple() {
    let source = resource_content(true, &["collection"], "tuple.mamba");
    to_py!(source);
}
