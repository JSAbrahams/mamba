use mamba::check::check_all;
use mamba::parse::parse;

use crate::common::resource_content;

#[test]
fn list_builder_illegal_op_cond() {
    let source = resource_content(false, &["type", "collection"], "list_builder_illegal_op_cond.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn list_builder_illegal_op_not_bool() {
    let source = resource_content(false, &["type", "collection"], "list_builder_illegal_op_not_bool.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn list_builder_nested_undefined_var() {
    let source = resource_content(false, &["type", "collection"], "list_builder_nested_undefined_var.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn list_builder_with_cond_not_boolean() {
    let source = resource_content(false, &["type", "collection"], "list_builder_with_cond_not_boolean.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn list_conflicting_collection_types() {
    let source = resource_content(false, &["type", "collection"], "list_conflicting_collection_types.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn list_builder_with_undefined_var() {
    let source = resource_content(false, &["type", "collection"], "list_builder_with_undefined_var.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn list_builder_with_no_expr() {
    let source = resource_content(false, &["type", "collection"], "list_builder_with_no_expr.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn set_builder_illegal_op_cond() {
    let source = resource_content(false, &["type", "collection"], "set_builder_illegal_op_cond.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn set_builder_illegal_op_not_bool() {
    let source = resource_content(false, &["type", "collection"], "set_builder_illegal_op_not_bool.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn set_builder_nested_undefined_var() {
    let source = resource_content(false, &["type", "collection"], "set_builder_nested_undefined_var.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn set_builder_with_cond_not_boolean() {
    let source = resource_content(false, &["type", "collection"], "set_builder_with_cond_not_boolean.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn set_conflicting_collection_types() {
    let source = resource_content(false, &["type", "collection"], "set_not_subscriptable.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn set_builder_with_undefined_var() {
    let source = resource_content(false, &["type", "collection"], "set_builder_with_undefined_var.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn set_builder_with_no_expr() {
    let source = resource_content(false, &["type", "collection"], "set_builder_with_no_expr.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn set_no_get() {
    let source = resource_content(false, &["type", "collection"], "set_no_get.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn set_not_subscriptable() {
    let source = resource_content(false, &["type", "collection"], "set_not_subscriptable.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}
