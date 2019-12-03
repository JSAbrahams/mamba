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
fn assign_to_function_call() {
    let source = resource_content(false, &["type", "definition"], "assign_to_function_call.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn assign_to_inner_non_mut() {
    let source = resource_content(false, &["type", "definition"], "assign_to_inner_non_mut.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn assign_to_inner_non_mut2() {
    let source = resource_content(false, &["type", "definition"], "assign_to_inner_non_mut2.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
#[ignore]
fn assign_to_inner_non_mut3() {
    let source = resource_content(false, &["type", "definition"], "assign_to_inner_non_mut3.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn undefined_variable() {
    let source = resource_content(false, &["type", "definition"], "undefined_variable.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn nested_function_private_field() {
    let source =
        resource_content(false, &["type", "definition"], "nested_function_private_field.mamba");
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
fn non_mutable_in_call_chain() {
    let source =
        resource_content(false, &["type", "definition"], "non_mutable_in_call_chain.mamba");
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
