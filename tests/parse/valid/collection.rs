use mamba::lex::tokenize;
use mamba::parse::parse;
use mamba::parse::result::ParseResult;

use crate::common::*;

#[test]
fn list_expression() -> ParseResult<()> {
    let source = resource_content(true, &["collection"], "tuple.mamba");
    parse(&tokenize(&source).unwrap()).map(|_| ())
}

#[test]
#[ignore]
fn parse_map() -> ParseResult<()> {
    let source = resource_content(true, &["collection"], "map.mamba");
    parse(&tokenize(&source).unwrap()).map(|_| ())
}

#[test]
fn parse_set() -> ParseResult<()> {
    let source = resource_content(true, &["collection"], "set.mamba");
    parse(&tokenize(&source).unwrap()).map(|_| ())
}

#[test]
fn parse_tuple() -> ParseResult<()> {
    let source = resource_content(true, &["collection"], "tuple.mamba");
    parse(&tokenize(&source).unwrap()).map(|_| ())
}
