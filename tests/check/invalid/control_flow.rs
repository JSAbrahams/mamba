use mamba::check::check_all;
use mamba::parse::parse;

use crate::common::resource_content;

#[test]
fn float_and() {
    let source = resource_content(false, &["type", "control_flow"], "float_and.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn for_non_iterable() {
    let source = resource_content(false, &["type", "control_flow"], "for_non_iterable.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn for_over_union_one_not_minus() {
    let source = resource_content(false, &["type", "control_flow"], "for_over_union_one_not_minus.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn if_not_bool_union() {
    let source = resource_content(false, &["type", "control_flow"], "if_not_bool_union.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn if_not_boolean() {
    let source = resource_content(false, &["type", "control_flow"], "if_not_boolean.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn not_integer() {
    let source = resource_content(false, &["type", "control_flow"], "not_integer.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn or_float() {
    let source = resource_content(false, &["type", "control_flow"], "or_float.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}
