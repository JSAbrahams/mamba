use crate::check::{check_test, CheckTestRet};
use crate::common::resource_content;

#[test]
fn exception_and_type() -> CheckTestRet {
    let source = resource_content(true, &["function"], "exception_and_type.mamba");
    check_test(&source)
}

#[test]
fn allowed_exception() -> CheckTestRet {
    let source = resource_content(true, &["function"], "allowed_exception.mamba");
    check_test(&source)
}

#[test]
fn call_mut_function() -> CheckTestRet {
    let source = resource_content(true, &["function"], "call_mut_function.mamba");
    check_test(&source)
}

#[test]
fn allowed_pass() -> CheckTestRet {
    let source = resource_content(true, &["function"], "allowed_pass.mamba");
    check_test(&source)
}
