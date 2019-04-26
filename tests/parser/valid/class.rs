use crate::util::*;
use mamba::lexer::tokenize;
use mamba::parser::ast::*;
use mamba::parser::parse;
use mamba::parser::parse_direct;

#[test]
fn parse_class() {
    let source = valid_resource_content(&["class"], "class.mamba");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_imports_class() {
    let source = valid_resource_content(&["class"], "import.mamba");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}
