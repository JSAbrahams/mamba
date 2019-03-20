use crate::util::*;
use mamba::lexer::tokenize;
use mamba::parser::parse;

#[test]
fn parse_list() {
    let source = valid_resource_content(&["collection"],"tuple.mamba");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_map() {
    let source = valid_resource_content(&["collection"],"map.mamba");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_set() {
    let source = valid_resource_content(&["collection"],"set.mamba");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_tuple() {
    let source = valid_resource_content(&["collection"],"tuple.mamba");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}
