use mamba::check::check_all;
use mamba::lex::tokenize;
use mamba::parse::parse;

use crate::common::resource_content;

#[test]
#[ignore]
fn all_mutable_in_call_chain() {
    let source = resource_content(true, &["definition"], "all_mutable_in_call_chain.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap();
}

#[test]
#[ignore]
fn nested_mut_field() {
    let source = resource_content(true, &["definition"], "nested_mut_field.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap();
}

#[test]
#[ignore]
fn assign_to_inner_mut() {
    let source = resource_content(true, &["definition"], "assign_to_inner_mut.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap();
}

#[test]
#[ignore]
fn nested_function() {
    let source = resource_content(true, &["definition"], "nested_function.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap();
}

#[test]
#[ignore] // Ignore tuples for now
fn tuple_modify_mut() {
    let source = resource_content(true, &["definition"], "tuple_modify_mut.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap();
}

#[test]
#[ignore] // Ignore tuples for now
fn tuple_modify_outer_mut() {
    let source = resource_content(true, &["definition"], "tuple_modify_outer_mut.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap();
}

#[test]
#[ignore]
fn f_strings() {
    let source = resource_content(true, &["definition"], "f_strings.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap();
}
