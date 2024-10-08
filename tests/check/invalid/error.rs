use mamba::check::check_all;
use mamba::parse::ast::AST;

use crate::common::resource_content;

#[test]
fn handle_only_id() {
    let source = resource_content(false, &["type", "error"], "handle_only_id.mamba");
    check_all(&[source.parse::<AST>().unwrap()]).unwrap_err();
}

#[test]
fn unhandled_exception() {
    let source = resource_content(false, &["type", "error"], "unhandled_exception.mamba");
    check_all(&[source.parse::<AST>().unwrap()]).unwrap_err();
}

#[test]
fn using_old_resource_in_with() {
    let source = resource_content(
        false,
        &["type", "error"],
        "using_old_resource_in_with.mamba",
    );
    check_all(&[source.parse::<AST>().unwrap()]).unwrap_err();
}

#[test]
fn with_wrong_type() {
    let source = resource_content(false, &["type", "error"], "with_wrong_type.mamba");
    check_all(&[source.parse::<AST>().unwrap()]).unwrap_err();
}

#[test]
fn with_not_expression() {
    let source = resource_content(false, &["type", "error"], "with_not_expression.mamba");
    check_all(&[source.parse::<AST>().unwrap()]).unwrap_err();
}
