use mamba::check::check_all;
use mamba::parse::parse;

use crate::common::resource_content;

#[test]
fn builder_illegal_op_cond() {
    let source = resource_content(false, &["type", "collection"], "builder_illegal_op_cond.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn builder_illegal_op_not_bool() {
    let source = resource_content(false, &["type", "collection"], "builder_illegal_op_not_bool.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn builder_with_cond_not_boolean() {
    let source = resource_content(false, &["type", "collection"], "builder_with_cond_not_boolean.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn conflicting_collection_types() {
    let source = resource_content(false, &["type", "collection"], "conflicting_collection_types.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn builder_with_undefined_var() {
    let source = resource_content(false, &["type", "collection"], "builder_with_undefined_var.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn builder_with_no_expr() {
    let source = resource_content(false, &["type", "collection"], "builder_with_no_expr.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}
