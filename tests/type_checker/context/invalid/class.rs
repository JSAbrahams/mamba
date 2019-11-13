use mamba::lexer::tokenize;
use mamba::parser::parse;
use mamba::type_checker::check_all;

use crate::common::resource_content;

#[test]
fn class_with_args_and_init() {
    let source = resource_content(false, &["type", "class"], "args_and_init.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn generic_unknown_type() {
    let source = resource_content(false, &["type", "class"], "generic_unknown_type.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn incompat_parent_generic() {
    let source = resource_content(false, &["type", "class"], "incompat_parent_generic.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn no_generic_arg() {
    let source = resource_content(false, &["type", "class"], "no_generic_arg.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn wrong_generic_type() {
    let source = resource_content(false, &["type", "class"], "wrong_generic_type.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}
