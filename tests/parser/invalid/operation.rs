use mamba::lexer::token::Token::*;
use mamba::lexer::tokenize;
use mamba::parser::ast::ASTNode;
use mamba::parser::parse_direct;

#[test]
fn addition_missing_factor() {
    let source = String::from("a +");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn subtraction_missing_factor() {
    let source = String::from("b -");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn multiplication_missing_factor() {
    let source = String::from("b *");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn division_missing_factor() {
    let source = String::from("b /");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn power_missing_factor() {
    let source = String::from("asd ^");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn mod_missing_factor() {
    let source = String::from("y mod");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn is_missing_value_left() {
    let source = String::from("is a");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn is_missing_value_right() {
    let source = String::from("kotlin is");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn isnt_missing_value_left() {
    let source = String::from("isnt a");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn isnt_missing_value_right() {
    let source = String::from("kotlin isnt");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn isa_missing_value_left() {
    let source = String::from("isa a");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn isa_missing_value_right() {
    let source = String::from("kotlin isa");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn isnta_missing_value_left() {
    let source = String::from("isnta a");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn isnta_missing_value_right() {
    let source = String::from("kotlin isnta");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn equality_missing_value_left() {
    let source = String::from("= a");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn equality_missing_value_right() {
    let source = String::from("kotlin =");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn le_missing_value_left() {
    let source = String::from("< a");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn le_missing_value_right() {
    let source = String::from("kotlin <");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn leq_missing_value_left() {
    let source = String::from("<= a");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn leq_missing_value_right() {
    let source = String::from("kotlin <=");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn ge_missing_value_left() {
    let source = String::from("> a");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn ge_missing_value_right() {
    let source = String::from("kotlin >");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn geq_missing_value_left() {
    let source = String::from(">= a");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn geq_missing_value_right() {
    let source = String::from("kotlin >=");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn and_missing_value_left() {
    let source = String::from("and a");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn and_missing_value_right() {
    let source = String::from("kotlin and");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn or_missing_value_left() {
    let source = String::from("or a");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn or_missing_value_right() {
    let source = String::from("kotlin or");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn not_missing_value() {
    let source = String::from("not");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn sqrt_missing_value() {
    let source = String::from("sqrt");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}