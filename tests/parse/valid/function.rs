use mamba::lex::tokenize;
use mamba::parse::parse;
use mamba::parse::result::ParseResult;

use crate::common::*;

#[test]
fn function_definitions() -> ParseResult<()> {
    let source = resource_content(true, &["function"], "definition.mamba");
    parse(&tokenize(&source).unwrap()).map(|_| ())
}

#[test]
fn function_calling() -> ParseResult<()> {
    let source = resource_content(true, &["function"], "calls.mamba");
    parse(&tokenize(&source).unwrap()).map(|_| ())
}
