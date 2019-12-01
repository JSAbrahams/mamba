use crate::common::resource_content;
use mamba::lexer::tokenize;
use mamba::parser::parse;
use mamba::type_checker::check_all;

#[test]
fn nested_mut_field() {
    let source = resource_content(true, &["definition"], "nested_mut_field.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap();
}

#[test]
fn nested_function() {
    let source = resource_content(true, &["definition"], "nested_function.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap();
}

#[test]
fn tuple_modify_mut() {
    let source = resource_content(true, &["definition"], "tuple_modify_mut.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap();
}

#[test]
fn tuple_modify_outer_mut() {
    let source = resource_content(true, &["definition"], "tuple_modify_outer_mut.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap();
}

#[test]
fn f_strings() {
    let source = resource_content(true, &["definition"], "f_strings.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap();
}
