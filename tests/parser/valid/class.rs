use crate::common::*;
use mamba::lexer::tokenize;
use mamba::parser::parse;

#[test]
fn parse_class() {
    let source = resource_content(true, &["class"], "class.mamba");
    parse(&tokenize(&source).unwrap()).unwrap();
}

#[test]
fn parse_imports_class() {
    let source = resource_content(true, &["class"], "import.mamba");
    parse(&tokenize(&source).unwrap()).unwrap();
}
