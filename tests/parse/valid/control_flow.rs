use mamba::lex::tokenize;
use mamba::parse::parse;
use mamba::parse::result::ParseResult;

use crate::common::resource_content;

#[test]
fn while_statements() -> ParseResult<()> {
    let source = resource_content(true, &["control_flow"], "while.mamba");
    parse(&tokenize(&source).unwrap()).map(|_| ())
}
