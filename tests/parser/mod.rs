use crate::util::*;
use mamba::lexer::tokenize;
use mamba::parser::parse;

pub mod class;
pub mod collection;
pub mod compound;
pub mod control_flow;
pub mod function;

#[test]
fn parse_empty_file() {
    let source = valid_resource_content(&[], "empty_file.mamba");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}
