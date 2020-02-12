use crate::common::resource_content;
use mamba::lexer::tokenize;
use mamba::parser::parse;
use mamba::type_checker::check_all;

#[test]
fn exception_and_type() {
    let source = resource_content(true, &["function"], "exception_and_type.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap();
}

#[test]
fn allowed_exception() {
    let source = resource_content(true, &["function"], "allowed_exception.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap();
}

#[test]
fn call_mut_function() {
    let source = resource_content(true, &["function"], "call_mut_function.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap();
}

#[test]
fn allowed_pass() {
    let source = resource_content(true, &["function"], "allowed_pass.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap();
}
