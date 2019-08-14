use mamba::lexer::tokenize;
use mamba::parser::parse;

#[test]
fn for_missing_do() {
    let source = String::from("for a in c d");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn for_missing_body() {
    let source = String::from("for a in c");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn if_missing_then() {
    let source = String::from("if a b");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn if_missing_body() {
    let source = String::from("if a then");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn if_then_missing_body() {
    let source = String::from("if a then b else");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn match_missing_condition() {
    let source = String::from("match\n    a => b");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn match_missing_arms() {
    let source = String::from("match a with\n    ");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn match_missing_arms_no_newline() {
    let source = String::from("match a");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn while_missing_condition() {
    let source = String::from("while do b");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn while_missing_body() {
    let source = String::from("while a do");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn while_missing_do() {
    let source = String::from("while a b");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}
