use mamba::check::check_all;
use mamba::parse::parse;

use crate::common::resource_content;

#[test]
#[ignore] // No way yet to define errors for std/primitives, see #392
fn dictionary_key_error_not_caught() {
    let source = resource_content(false, &["type", "collection"], "dictionary_assume_not_optional.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn dictionary_in_fun_wrong_ret_ty() {
    let source = resource_content(false, &["type", "collection"], "dictionary_in_fun_wrong_ret_ty.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn dictionary_not_sliceable() {
    let source = resource_content(false, &["type", "collection"], "dictionary_not_sliceable.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn dictionary_use_value_as_other_type() {
    let source = resource_content(false, &["type", "collection"], "dictionary_use_value_as_other_type.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn dictionary_wrong_key_type() {
    let source = resource_content(false, &["type", "collection"], "dictionary_wrong_key_type.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

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
fn set_behind_function_not_subscriptable() {
    let source = resource_content(false, &["type", "collection"], "set_behind_function_not_subscriptable.mamba");
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
