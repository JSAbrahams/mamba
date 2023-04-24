use mamba::check::check_all;
use mamba::parse::ast::AST;

use crate::common::resource_content;

#[test]
fn in_dict_wrong_ty() {
    let source = resource_content(false, &["type", "operation"], "in_dict_wrong_ty.mamba");
    check_all(&[source.parse::<AST>().unwrap()]).unwrap_err();
}

#[test]
fn in_list_wrong_ty() {
    let source = resource_content(false, &["type", "operation"], "in_list_wrong_ty.mamba");
    check_all(&[source.parse::<AST>().unwrap()]).unwrap_err();
}

#[test]
fn in_set_wrong_ty() {
    let source = resource_content(false, &["type", "operation"], "in_set_wrong_ty.mamba");
    check_all(&[source.parse::<AST>().unwrap()]).unwrap_err();
}

#[test]
fn isa_not_id() {
    let source = resource_content(false, &["type", "operation"], "isa_not_id.mamba");
    check_all(&[source.parse::<AST>().unwrap()]).unwrap_err();
}

#[test]
fn reassign_to_nullable() {
    let source = resource_content(false, &["type", "operation"], "reassign_to_nullable.mamba");
    check_all(&[source.parse::<AST>().unwrap()]).unwrap_err();
}

#[test]
fn reassign_to_undefined() {
    let source = resource_content(false, &["type", "operation"], "reassign_to_undefined.mamba");
    check_all(&[source.parse::<AST>().unwrap()]).unwrap_err();
}

#[test]
fn string_minus() {
    let source = resource_content(false, &["type", "operation"], "string_minus.mamba");
    check_all(&[source.parse::<AST>().unwrap()]).unwrap_err();
}

#[test]
fn undefined_field_fstring() {
    let source = resource_content(false, &["type", "operation"], "undefined_field_fstring.mamba");
    check_all(&[source.parse::<AST>().unwrap()]).unwrap_err();
}
