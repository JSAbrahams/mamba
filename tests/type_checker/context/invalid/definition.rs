use crate::common::resource_content;
use mamba::lexer::tokenize;
use mamba::parser::parse;
use mamba::type_checker::check_all;

#[test]
fn assign_wrong_type() {
    let source = resource_content(false, &["type", "definition"], "assign_wrong_type.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn undefined_variable() {
    let source = resource_content(false, &["type", "definition"], "undefined_variable.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn nested_non_mut_field() {
    let source = resource_content(false, &["type", "definition"], "nested_non_mut_field.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn reassign_non_mut() {
    let source = resource_content(false, &["type", "definition"], "reassign_non_mut.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn reassign_non_mut_field() {
    let source = resource_content(false, &["type", "definition"], "reassign_non_mut_field.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn tuple_modify_mut() {
    let source = resource_content(false, &["type", "definition"], "tuple_modify_mut.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}
