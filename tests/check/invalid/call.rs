use mamba::check::check_all;
use mamba::parse::parse;

use crate::common::resource_content;

#[test]
fn access_list_with_string() {
    let source = resource_content(false, &["type", "call"], "call_with_parent.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn calls_wrong_primitive() {
    let source = resource_content(false, &["type", "call"], "calls_wrong_primitive.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn calls_wrong_type() {
    let source = resource_content(false, &["type", "call"], "calls_wrong_type.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}
