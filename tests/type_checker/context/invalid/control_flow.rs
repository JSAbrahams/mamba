use crate::common::resource_content;
use mamba::lexer::tokenize;
use mamba::parser::parse;
use mamba::type_checker::check_all;

#[test]
fn float_and() {
    let source = resource_content(false, &["type", "control_flow"], "float_and.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn for_non_iterable() {
    let source = resource_content(false, &["type", "control_flow"], "for_non_iterable.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn if_boolean() {
    let source = resource_content(false, &["type", "control_flow"], "if_not_boolean.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn not_integer() {
    let source = resource_content(false, &["type", "control_flow"], "not_integer.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn or_float() {
    let source = resource_content(false, &["type", "control_flow"], "or_float.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}
