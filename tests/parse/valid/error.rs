use mamba::lex::tokenize;
use mamba::parse::parse;
use mamba::parse::result::ParseResult;

use crate::common::resource_content;

#[test]
fn handle_verify() -> ParseResult<()> {
    let source = resource_content(true, &["error"], "handle.mamba");
    parse(&tokenize(&source).unwrap()).map(|_| ())
}

#[test]
fn raises_verify() -> ParseResult<()> {
    let source = resource_content(true, &["error"], "raise.mamba");
    parse(&tokenize(&source).unwrap()).map(|_| ())
}

#[test]
fn with_verify() -> ParseResult<()> {
    let source = resource_content(true, &["error"], "with.mamba");
    parse(&tokenize(&source).unwrap()).map(|_| ())
}
