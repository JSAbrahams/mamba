use mamba::check::check_all;
use mamba::parse::parse;

use crate::common::resource_content;

#[test]
fn access_list_with_string() {
    let source = resource_content(false, &["type", "access"], "access_list_with_string.mamba");
    check_all(&[(*parse(&source).unwrap(), None, None)]).unwrap_err();
}

#[test]
#[ignore] // Cannot parse dictionaries yet
fn access_string_dict_with_int() {
    let source = resource_content(false, &["type", "access"], "access_string_dict_with_int.mamba");
    check_all(&[(*parse(&source).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn slice_begin_wrong_type() {
    let source = resource_content(false, &["type", "access"], "slice_begin_wrong_type.mamba");
    check_all(&[(*parse(&source).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn slice_end_wrong_type() {
    let source = resource_content(false, &["type", "access"], "slice_end_wrong_type.mamba");
    check_all(&[(*parse(&source).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn slice_step_wrong_type() {
    let source = resource_content(false, &["type", "access"], "slice_step_wrong_type.mamba");
    check_all(&[(*parse(&source).unwrap(), None, None)]).unwrap_err();
}
