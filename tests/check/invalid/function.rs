use mamba::check::check_all;
use mamba::parse::parse;

use crate::common::resource_content;

#[test]
fn outside_class_with_self() {
    let source = resource_content(false, &["type", "function"], "outside_class_with_self.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn incompatible_types() {
    let source = resource_content(false, &["type", "function"], "incompatible_types.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn no_enough_arg() {
    let source = resource_content(false, &["type", "function"], "no_enough_arg.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn no_enough_arg_with_default() {
    let source = resource_content(false, &["type", "function"], "not_enough_arg_with_default.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn arg_no_type() {
    let source = resource_content(false, &["type", "function"], "arg_no_type.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn as_statement() {
    let source = resource_content(false, &["type", "function"], "as_statement.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn return_illegal() {
    let source = resource_content(false, &["type", "function"], "return_illegal.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn statement_as_param() {
    let source = resource_content(false, &["type", "function"], "statement_as_param.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn too_many_arg() {
    let source = resource_content(false, &["type", "function"], "too_many_arg.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn too_many_arg_with_default() {
    let source = resource_content(false, &["type", "function"], "too_many_arg_with_default.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn unexpected_pass() {
    let source = resource_content(false, &["type", "function"], "unexpected_pass.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn unmentioned_exception() {
    let source = resource_content(false, &["type", "function"], "unmentioned_exception.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn wrong_exception() {
    let source = resource_content(false, &["type", "function"], "wrong_exception.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
#[ignore]  // must construct system which identifies exit points in function
fn function_with_stmt_body() {
    let source = resource_content(false, &["type", "function"], "function_with_stmt_body.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn function_with_stmt_body_ret() {
    let source = resource_content(false, &["type", "function"], "function_with_stmt_body_ret.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn return_exp_expr() {
    let source = resource_content(false, &["type", "function"], "return_exp_expr.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn wrong_return_type() {
    let source = resource_content(false, &["type", "function"], "wrong_return_type.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
fn return_undefined() {
    let source = resource_content(false, &["type", "function"], "return_undefined.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

#[test]
#[ignore] // Ignore mutability for now
fn call_mut_function() {
    let source = resource_content(false, &["type", "function"], "call_mut_function.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}
