use mamba::check::check_all;
use mamba::parse::parse;

use crate::common::resource_content;

#[test]
fn reassign_non_existent() {
    let source = resource_content(false, &["type", "class"], "reassign_non_existent.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn assign_to_non_existent_self() {
    let source = resource_content(false, &["type", "class"], "assign_to_non_existent_self.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn access_unassigned_field() {
    let source = resource_content(false, &["type", "class"], "access_unassigned_field.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn reassign_wrong_type() {
    let source = resource_content(false, &["type", "class"], "reassign_wrong_type.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn reassign_function() {
    let source = resource_content(false, &["type", "class"], "reassign_function.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn access_field_wrong_type() {
    let source = resource_content(false, &["type", "class"], "access_field_wrong_type.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn access_function_wrong_type() {
    let source = resource_content(false, &["type", "class"], "access_function_wrong_type.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn class_with_args_and_init() {
    let source = resource_content(false, &["type", "class"], "args_and_init.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn assign_to_inner_inner_not_allowed() {
    let source =
        resource_content(false, &["type", "class"], "assign_to_inner_inner_not_allowed.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn assign_to_inner_not_allowed() {
    let source = resource_content(false, &["type", "class"], "assign_to_inner_not_allowed.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
#[ignore]
fn generic_unknown_type() {
    let source = resource_content(false, &["type", "class"], "generic_unknown_type.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
#[ignore]
fn incompat_parent_generic() {
    let source = resource_content(false, &["type", "class"], "incompat_parent_generic.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
#[ignore]
fn no_generic_arg() {
    let source = resource_content(false, &["type", "class"], "no_generic_arg.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn one_tuple_not_assigned_to() {
    let source = resource_content(false, &["type", "class"], "one_tuple_not_assigned_to.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
#[ignore] // need to fix
fn reassign_to_unassigned_class_var() {
    let source = resource_content(false, &["type", "class"], "reassign_to_unassigned_class_var.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn top_level_class_not_assigned_to() {
    let source =
        resource_content(false, &["type", "class"], "top_level_class_not_assigned_to.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
#[ignore]
fn wrong_generic_type() {
    let source = resource_content(false, &["type", "class"], "wrong_generic_type.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}
