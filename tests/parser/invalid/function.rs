use crate::util::*;
use mamba::lexer::tokenize;
use mamba::parser::parse_direct;

#[test]
fn direct_call_missing_closing_bracket() {
    let source = String::from("a(b");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn regular_call_missing_closing_bracket() {
    let source = String::from("instance.a(b");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}
