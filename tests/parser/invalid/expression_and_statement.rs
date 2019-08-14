use mamba::lexer::tokenize;
use mamba::parser::parse;

#[test]
fn print_missing_arg() {
    let source = String::from("print");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn range_missing_from() {
    let source = String::from(".. b");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn range_inc_missing_from() {
    let source = String::from("..= b");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn range_missing_to() {
    let source = String::from("a ..");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn range_incl_missing_to() {
    let source = String::from("a ..=");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn reassign_missing_value() {
    let source = String::from("a <-");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn quest_or_missing_alternative() {
    let source = String::from("a ?or");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn quest_or_on_nothing() {
    let source = String::from("?or");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}
