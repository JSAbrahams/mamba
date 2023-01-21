use mamba::check::check_all;
use mamba::parse::parse;

use crate::common::resource_content;

#[test]
fn access_list_with_string() {
    let source = resource_content(false, &["type", "access"], "access_list_with_string.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn access_set() {
    let source = resource_content(false, &["type", "access"], "access_set.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
#[ignore] // Cannot parse dictionaries yet
fn access_string_dict_with_int() {
    let source = resource_content(false, &["type", "access"], "access_string_dict_with_int.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn slice_begin_wrong_type() {
    let source = resource_content(false, &["type", "access"], "slice_begin_wrong_type.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn empty_define_f() {
    let source = resource_content(false, &["type", "access"], "empty_define_f.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn empty_define_field() {
    let source = resource_content(false, &["type", "access"], "empty_define_field.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn empty_define_str() {
    let source = resource_content(false, &["type", "access"], "empty_define_str.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn slice_end_wrong_type() {
    let source = resource_content(false, &["type", "access"], "slice_end_wrong_type.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn slice_step_wrong_type() {
    let source = resource_content(false, &["type", "access"], "slice_step_wrong_type.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn access_int() {
    let source = resource_content(false, &["type", "access"], "access_int.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}
