use mamba::check::check_all;
use mamba::parse::ast::AST;

use crate::common::resource_content;

#[test]
fn access_match_arms_variable() {
    let source = resource_content(
        false,
        &["type", "control_flow"],
        "access_match_arms_variable.mamba",
    );
    check_all(&[source.parse::<AST>().unwrap()]).unwrap_err();
}

#[test]
fn class_field_assigned_to_only_one_arm_match() {
    let source = resource_content(
        false,
        &["type", "control_flow"],
        "class_field_assigned_to_only_one_arm_match.mamba",
    );
    check_all(&[source.parse::<AST>().unwrap()]).unwrap_err();
}

#[test]
fn class_field_assigned_to_only_then() {
    let source = resource_content(
        false,
        &["type", "control_flow"],
        "class_field_assigned_to_only_then.mamba",
    );
    check_all(&[source.parse::<AST>().unwrap()]).unwrap_err();
}

#[test]
fn class_field_assigned_to_only_else() {
    let source = resource_content(
        false,
        &["type", "control_flow"],
        "class_field_assigned_to_only_else.mamba",
    );
    check_all(&[source.parse::<AST>().unwrap()]).unwrap_err();
}

#[test]
fn float_and() {
    let source = resource_content(false, &["type", "control_flow"], "float_and.mamba");
    check_all(&[source.parse::<AST>().unwrap()]).unwrap_err();
}

#[test]
fn for_non_iterable() {
    let source = resource_content(false, &["type", "control_flow"], "for_non_iterable.mamba");
    check_all(&[source.parse::<AST>().unwrap()]).unwrap_err();
}

#[test]
fn for_over_union_one_not_minus() {
    let source = resource_content(
        false,
        &["type", "control_flow"],
        "for_over_union_one_not_minus.mamba",
    );
    check_all(&[source.parse::<AST>().unwrap()]).unwrap_err();
}

#[test]
fn if_not_bool_union() {
    let source = resource_content(false, &["type", "control_flow"], "if_not_bool_union.mamba");
    check_all(&[source.parse::<AST>().unwrap()]).unwrap_err();
}

#[test]
fn if_not_boolean() {
    let source = resource_content(false, &["type", "control_flow"], "if_not_boolean.mamba");
    check_all(&[source.parse::<AST>().unwrap()]).unwrap_err();
}

#[test]
fn not_integer() {
    let source = resource_content(false, &["type", "control_flow"], "not_integer.mamba");
    check_all(&[source.parse::<AST>().unwrap()]).unwrap_err();
}

#[test]
fn or_float() {
    let source = resource_content(false, &["type", "control_flow"], "or_float.mamba");
    check_all(&[source.parse::<AST>().unwrap()]).unwrap_err();
}

#[test]
fn undefined_var_in_match_arm() {
    let source = resource_content(
        false,
        &["type", "control_flow"],
        "undefined_var_in_match_arm.mamba",
    );
    check_all(&[source.parse::<AST>().unwrap()]).unwrap_err();
}

#[test]
fn variable_defined_in_then() {
    let source = resource_content(
        false,
        &["type", "control_flow"],
        "variable_defined_in_then.mamba",
    );
    check_all(&[source.parse::<AST>().unwrap()]).unwrap_err();
}

#[test]
fn variable_defined_in_else() {
    let source = resource_content(
        false,
        &["type", "control_flow"],
        "variable_defined_in_else.mamba",
    );
    check_all(&[source.parse::<AST>().unwrap()]).unwrap_err();
}
