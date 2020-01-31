use crate::common::resource_content;
use mamba::lexer::tokenize;
use mamba::parser::parse;
use mamba::type_checker::check_all;

#[test]
fn old_resource_with() {
    let source = resource_content(false, &["type", "error"], "using_old_resource_in_with.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn wrong_type_with() {
    let source = resource_content(false, &["type", "error"], "with_wrong_type.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn with_not_expression() {
    let source = resource_content(false, &["type", "error"], "with_not_expression.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}
