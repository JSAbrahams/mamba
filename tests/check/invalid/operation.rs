use mamba::check::check_all;
use mamba::parse::parse;

use crate::common::resource_content;

#[test]
fn reassign_to_nullable() {
    let source = resource_content(false, &["type", "operation"], "reassign_to_nullable.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn reassign_to_undefined() {
    let source = resource_content(false, &["type", "operation"], "reassign_to_undefined.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn string_minus() {
    let source = resource_content(false, &["type", "operation"], "string_minus.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn undefined_field_fstring() {
    let source = resource_content(false, &["type", "operation"], "undefined_field_fstring.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}
