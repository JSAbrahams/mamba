use crate::util::*;
use mamba::lexer::tokenize;
use mamba::parser::parse_direct;

#[test]
fn print_missing_arg() {
    let source = String::from("print");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn range_missing_from() {
    let source = String::from(".. b");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn range_inc_missing_from() {
    let source = String::from("..= b");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn range_missing_to() {
    let source = String::from("a ..");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn range_incl_missing_to() {
    let source = String::from("a ..=");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn rassign_missing_value() {
    let source = String::from("a <-");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn quest_or_missing_alternative() {
    let source = String::from("a ?or");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}
