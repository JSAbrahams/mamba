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
fn function_return_undefined() {
    let source =
        resource_content(false, &["type", "definition"], "function_return_undefined.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn function_wrong_return() {
    let source = resource_content(false, &["type", "definition"], "function_wrong_return.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}
