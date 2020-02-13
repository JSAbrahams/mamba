use crate::common::*;
use mamba::lex::tokenize;
use mamba::parse::parse;

pub mod invalid;
pub mod valid;

#[test]
fn parse_empty_file() {
    let source = resource_content(true, &[], "empty_file.mamba");
    parse(&tokenize(&source).unwrap()).unwrap();
}
