use mamba::parse::lex::tokenize;
use mamba::parse::parse;
use mamba::parse::result::ParseResult;

use crate::common::*;

#[test]
fn parse_class() -> ParseResult<()> {
    let source = resource_content(true, &["class"], "types.mamba");
    parse(&tokenize(&source).unwrap()).map(|_| ())
}

#[test]
fn parse_imports_class() -> ParseResult<()> {
    let source = resource_content(true, &["class"], "import.mamba");
    parse(&tokenize(&source).unwrap()).map(|_| ())
}
