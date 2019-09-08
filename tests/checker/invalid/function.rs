use crate::common::resource_content;
use mamba::lexer::tokenize;
use mamba::parser::parse;
use mamba::type_checker::check_all;

#[test]
fn function_outside_class_with_self() {
    let source = resource_content(false, &["type", "function"], "no_class.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn function_argument_no_type() {
    let source = resource_content(false, &["type", "function"], "no_type.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}
