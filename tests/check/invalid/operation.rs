use mamba::check::check_all;
use mamba::parse::parse;

use crate::common::resource_content;

#[test]
fn string_minus() {
    let source = resource_content(false, &["type", "operation"], "string_minus.mamba");
    check_all(&[(*parse(&source).unwrap(), None, None)]).unwrap_err();
}

#[test]
fn undefined_field_fstring() {
    let source = resource_content(false, &["type", "operation"], "undefined_field_fstring.mamba");
    check_all(&[(*parse(&source).unwrap(), None, None)]).unwrap_err();
}
