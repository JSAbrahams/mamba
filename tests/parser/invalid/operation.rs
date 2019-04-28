use mamba::lexer::tokenize;
use mamba::parser::parse_direct;

#[test]
fn addition_missing_factor() {
    let source = String::from("a +");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn subtraction_missing_factor() {
    let source = String::from("b -");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn multiplication_missing_factor() {
    let source = String::from("b *");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn division_missing_factor() {
    let source = String::from("b /");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn power_missing_factor() {
    let source = String::from("asd ^");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn mod_missing_factor() {
    let source = String::from("y mod");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn is_missing_value_left() {
    let source = String::from("is a");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn is_missing_value_right() {
    let source = String::from("kotlin is");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn isnt_missing_value_left() {
    let source = String::from("isnt a");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn isnt_missing_value_right() {
    let source = String::from("kotlin isnt");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn isa_missing_value_left() {
    let source = String::from("isa a");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn isa_missing_value_right() {
    let source = String::from("kotlin isa");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn isnta_missing_value_left() {
    let source = String::from("isnta a");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn isnta_missing_value_right() {
    let source = String::from("kotlin isnta");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn equality_missing_value_left() {
    let source = String::from("= a");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn equality_missing_value_right() {
    let source = String::from("kotlin =");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn le_missing_value_left() {
    let source = String::from("< a");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn le_missing_value_right() {
    let source = String::from("kotlin <");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn leq_missing_value_left() {
    let source = String::from("<= a");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn leq_missing_value_right() {
    let source = String::from("kotlin <=");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn ge_missing_value_left() {
    let source = String::from("> a");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn ge_missing_value_right() {
    let source = String::from("kotlin >");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn geq_missing_value_left() {
    let source = String::from(">= a");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn geq_missing_value_right() {
    let source = String::from("kotlin >=");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn and_missing_value_left() {
    let source = String::from("and a");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn and_missing_value_right() {
    let source = String::from("kotlin and");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn or_missing_value_left() {
    let source = String::from("or a");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn or_missing_value_right() {
    let source = String::from("kotlin or");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn not_missing_value() {
    let source = String::from("not");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn sqrt_missing_value() {
    let source = String::from("sqrt");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}
