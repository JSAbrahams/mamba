use crate::common::resource_content;
use mamba::lexer::tokenize;
use mamba::parser::parse_direct;

#[test]
fn def_mut_private_wrong_order() {
    let source = String::from("def mut private a ");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn def_missing_id() {
    let source = String::from("def");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn def_fun_no_closing_brack() {
    let source = String::from("def f(a");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn def_fun_missing_arrow() {
    let source = String::from("def f(a) a * 10");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn def_fun_missing_brackets() {
    let source = String::from("def f => print a");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn type_annotation_in_tuple() {
    let source = resource_content(false, &["syntax"], "type_annotation_in_tuple.mamba");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}
