use crate::common::resource_content;
use mamba::check::check_all;
use mamba::lex::tokenize;
use mamba::parse::parse;

#[test]
fn outside_class_with_self() {
    let source = resource_content(false, &["type", "function"], "outside_class_with_self.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn incompatible_types() {
    let source = resource_content(false, &["type", "function"], "incompatible_types.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn arg_no_type() {
    let source = resource_content(false, &["type", "function"], "arg_no_type.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn as_statement() {
    let source = resource_content(false, &["type", "function"], "as_statement.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn return_illegal() {
    let source = resource_content(false, &["type", "function"], "return_illegal.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn statement_as_param() {
    let source = resource_content(false, &["type", "function"], "statement_as_param.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn unexpected_pass() {
    let source = resource_content(false, &["type", "function"], "unexpected_pass.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn unmentioned_exception() {
    let source = resource_content(false, &["type", "function"], "unmentioned_exception.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn wrong_exception() {
    let source = resource_content(false, &["type", "function"], "wrong_exception.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn wrong_return_type() {
    let source = resource_content(false, &["type", "function"], "wrong_return_type.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn return_undefined() {
    let source = resource_content(false, &["type", "function"], "return_undefined.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn call_mut_function() {
    let source = resource_content(false, &["type", "function"], "call_mut_function.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}
