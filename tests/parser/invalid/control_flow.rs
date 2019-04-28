use mamba::lexer::tokenize;
use mamba::parser::parse_direct;

#[test]
fn foreach_missing_in() {
    let source = String::from("foreach a c do d");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn foreach_missing_do() {
    let source = String::from("foreach a in c d");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn foreach_missing_body() {
    let source = String::from("foreach a in c");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn if_missing_then() {
    let source = String::from("if a b");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn if_missing_body() {
    let source = String::from("if a then");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn if_then_missing_body() {
    let source = String::from("if a then b else");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn match_missing_condition() {
    let source = String::from("match with\n    a => b");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn match_missing_arms() {
    let source = String::from("match a with\n    ");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn match_missing_arms_no_newline() {
    let source = String::from("match a with");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn while_missing_condition() {
    let source = String::from("while do b");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn while_missing_body() {
    let source = String::from("while a do");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn while_missing_do() {
    let source = String::from("while a b");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}
