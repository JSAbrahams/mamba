use mamba::lexer::tokenize;
use mamba::parser::parse_direct;

#[test]
fn foreach_missing_in() {
    let source = String::from("foreach a c do d");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn foreach_missing_do() {
    let source = String::from("foreach a in c d");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn foreach_missing_body() {
    let source = String::from("foreach a in c");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn if_missing_then() {
    let source = String::from("if a b");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn if_missing_body() {
    let source = String::from("if a then");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn if_then_missing_body() {
    let source = String::from("if a then b else");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn match_missing_condition() {
    let source = String::from("match with\n    a => b");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn match_missing_arms() {
    let source = String::from("match a with\n    ");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn match_missing_arms_no_newline() {
    let source = String::from("match a with");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn while_missing_condition() {
    let source = String::from("while do b");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn while_missing_body() {
    let source = String::from("while a do");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn while_missing_do() {
    let source = String::from("while a b");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}
