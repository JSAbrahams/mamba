use mamba::parse::lex::tokenize;
use mamba::parse::parse;
use mamba::parse::result::ParseResult;

use crate::common::*;

pub mod invalid;
pub mod valid;

#[test]
fn parse_empty_file() -> ParseResult<()> {
    let source = resource_content(true, &[], "empty_file.mamba");
    parse(&tokenize(&source).unwrap()).map(|_| ())
}
