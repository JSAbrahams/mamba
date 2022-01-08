use mamba::lex::tokenize;
use mamba::parse::parse;

use crate::common::*;

#[test]
fn parse_class() -> Result<(), String> {
    let source = resource_content(true, &["class"], "types.mamba");
    parse(&tokenize(&source).unwrap()).map_err(|e| format!("{}", e))?;
    Ok(())
}

#[test]
fn parse_imports_class() {
    let source = resource_content(true, &["class"], "import.mamba");
    parse(&tokenize(&source).unwrap()).unwrap();
}
