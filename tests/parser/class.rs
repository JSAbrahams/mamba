use crate::util::*;
use mamba::lexer::tokenize;
use mamba::parser::parse;

#[test]
fn parse_class() {
    let source = valid_resource_content(&["class"],"class.mamba");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}