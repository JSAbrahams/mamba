use mamba::lex::tokenize;
use mamba::parse::parse;

use crate::common::*;

#[test]
fn list_expression() {
    let source = resource_content(true, &["collection"], "tuple.mamba");
    parse(&tokenize(&source).unwrap()).unwrap();
}

#[test]
#[ignore]
fn parse_map() {
    let source = resource_content(true, &["collection"], "map.mamba");
    parse(&tokenize(&source).unwrap()).unwrap();
}

#[test]
fn parse_set() {
    let source = resource_content(true, &["collection"], "set.mamba");
    assert!(parse(&tokenize(&source).unwrap()).is_ok());
}

#[test]
fn parse_tuple() {
    let source = resource_content(true, &["collection"], "tuple.mamba");
    parse(&tokenize(&source).unwrap()).unwrap();
}
