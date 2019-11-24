use mamba::lexer::tokenize;
use mamba::parser::parse;
use mamba::type_checker::check_all;

use crate::common::resource_content;

// TODO implement generics fully in type checker

#[test]
fn access_private_field() {
    let source = resource_content(false, &["type", "class"], "access_private_field.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn access_private_function() {
    let source = resource_content(false, &["type", "class"], "access_private_function.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn type_def_with_private_field() {
    let source = resource_content(false, &["type", "class"], "type_def_with_private_field.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}
#[test]
fn class_with_args_and_init() {
    let source = resource_content(false, &["type", "class"], "args_and_init.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
#[ignore]
fn generic_unknown_type() {
    let source = resource_content(false, &["type", "class"], "generic_unknown_type.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
#[ignore]
fn incompat_parent_generic() {
    let source = resource_content(false, &["type", "class"], "incompat_parent_generic.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
#[ignore]
fn no_generic_arg() {
    let source = resource_content(false, &["type", "class"], "no_generic_arg.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
#[ignore]
fn wrong_generic_type() {
    let source = resource_content(false, &["type", "class"], "wrong_generic_type.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}
