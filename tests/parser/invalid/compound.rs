use crate::util::*;
use mamba::lexer::tokenize;
use mamba::parser::parse;

#[test]
fn parse_assigns_and_while() {
    let source = invalid_resource_content(&["syntax"], "assign_and_while.mamba");
    let err = parse(&tokenize(&source).unwrap());

    assert_eq!(err.is_err(), true);
}
