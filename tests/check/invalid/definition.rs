use mamba::check::check_all;
use mamba::parse::parse;

use crate::common::resource_content;

#[test]
fn argument_after_argument_with_default() {
    let source = resource_content(
        false,
        &["type", "definition"],
        "argument_after_argument_with_default.mamba",
    );
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn assign_wrong_type() {
    let source = resource_content(false, &["type", "definition"], "assign_wrong_type.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
#[ignore] // see #343
fn body_is_stmt() {
    let source = resource_content(false, &["type", "definition"], "body_is_stmt.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn function_ret_in_class_not_super() {
    let source = resource_content(false, &["type", "definition"], "function_ret_in_class_not_super.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn function_ret_not_super() {
    let source = resource_content(false, &["type", "definition"], "function_ret_not_super.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn assign_to_function_call() {
    let source = resource_content(false, &["type", "definition"], "assign_to_function_call.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
#[ignore] // Ignore mutability for now
fn assign_to_inner_non_mut() {
    let source = resource_content(false, &["type", "definition"], "assign_to_inner_non_mut.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn assign_to_inner_non_mut2() {
    let source = resource_content(false, &["type", "definition"], "assign_to_inner_non_mut2.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
#[ignore]
fn assign_to_inner_non_mut3() {
    let source = resource_content(false, &["type", "definition"], "assign_to_inner_non_mut3.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn undefined_variable() {
    let source = resource_content(false, &["type", "definition"], "undefined_variable.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
#[ignore] // Ignore mutability for now
fn nested_non_mut_field() {
    let source = resource_content(false, &["type", "definition"], "nested_non_mut_field.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
#[ignore] // Ignore mutability for now
fn reassign_non_mut() {
    let source = resource_content(false, &["type", "definition"], "reassign_non_mut.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn non_mutable_in_call_chain() {
    let source =
        resource_content(false, &["type", "definition"], "non_mutable_in_call_chain.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn raises_unmentioned_exception() {
    let source =
        resource_content(false, &["type", "definition"], "raises_unmentioned_exception.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn raises_non_exception() {
    let source = resource_content(false, &["type", "definition"], "raises_non_exception.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
#[ignore] // Ignore mutability for now
fn reassign_non_mut_field() {
    let source = resource_content(false, &["type", "definition"], "reassign_non_mut_field.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
#[ignore] // Ignore tuples for now
fn tuple_modify_inner_mut() {
    let source = resource_content(false, &["type", "definition"], "tuple_modify_inner_mut.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn tuple_modify_mut() {
    let source = resource_content(false, &["type", "definition"], "tuple_modify_mut.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn tuple_modify_mut_entire() {
    let source = resource_content(false, &["type", "definition"], "tuple_modify_mut_entire.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn tuple_assign_itself() {
    let source = resource_content(false, &["type", "definition"], "tuple_assign_itself.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn wrong_size_tuple() {
    let source = resource_content(false, &["type", "definition"], "wrong_size_tuple.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn wrong_size_tuple_2() {
    let source = resource_content(false, &["type", "definition"], "wrong_size_tuple_2.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn list_not_a_tuple() {
    let source = resource_content(false, &["type", "definition"], "list_not_a_tuple.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}
