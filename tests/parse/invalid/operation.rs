use mamba::lex::tokenize;
use mamba::parse::parse;

#[test]
fn addition_missing_factor() {
    let source = String::from("a +");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn subtraction_missing_factor() {
    let source = String::from("b -");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn multiplication_missing_factor() {
    let source = String::from("b *");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn division_missing_factor() {
    let source = String::from("b /");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn power_missing_factor() {
    let source = String::from("a ^");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn mod_missing_factor() {
    let source = String::from("y mod");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn is_missing_value_left() {
    let source = String::from("is a");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn is_missing_value_right() {
    let source = String::from("kotlin is");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn isnt_missing_value_left() {
    let source = String::from("isnt a");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn isnt_missing_value_right() {
    let source = String::from("kotlin isnt");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn isa_missing_value_left() {
    let source = String::from("isa a");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn isa_missing_value_right() {
    let source = String::from("kotlin isa");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn isnta_missing_value_left() {
    let source = String::from("isnta a");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn isnta_missing_value_right() {
    let source = String::from("kotlin isnta");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn equality_missing_value_left() {
    let source = String::from("= a");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn equality_missing_value_right() {
    let source = String::from("kotlin =");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn le_missing_value_left() {
    let source = String::from("< a");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn le_missing_value_right() {
    let source = String::from("kotlin <");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn leq_missing_value_left() {
    let source = String::from("<= a");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn leq_missing_value_right() {
    let source = String::from("kotlin <=");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn ge_missing_value_left() {
    let source = String::from("> a");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn ge_missing_value_right() {
    let source = String::from("kotlin >");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn geq_missing_value_left() {
    let source = String::from(">= a");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn geq_missing_value_right() {
    let source = String::from("kotlin >=");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn and_missing_value_left() {
    let source = String::from("and a");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn and_missing_value_right() {
    let source = String::from("kotlin and");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn or_missing_value_left() {
    let source = String::from("or a");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn or_missing_value_right() {
    let source = String::from("kotlin or");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn not_missing_value() {
    let source = String::from("not");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn sqrt_missing_value() {
    let source = String::from("sqrt");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}
