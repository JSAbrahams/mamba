use mamba::check::check_all;
use mamba::parse::parse;

use crate::common::resource_content;

#[test]
fn using_old_resource_in_with() {
    let source = resource_content(false, &["type", "error"], "using_old_resource_in_with.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn with_wrong_type() {
    let source = resource_content(false, &["type", "error"], "with_wrong_type.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn with_not_expression() {
    let source = resource_content(false, &["type", "error"], "with_not_expression.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}
