use crate::common::*;
use mamba::lexer::tokenize;
use mamba::parser::parse;

#[test]
fn parse_class() {
    let source = valid_resource_content(&["class"], "class.mamba");
    parse(&tokenize(&source).unwrap()).unwrap();
}

#[test]
fn parse_imports_class() {
    let source = valid_resource_content(&["class"], "import.mamba");
    parse(&tokenize(&source).unwrap()).unwrap();
}
