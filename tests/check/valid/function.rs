use crate::check::{check_test, CheckTestRet};
use crate::common::resource_content;

#[test]
fn exception_and_type() -> CheckTestRet {
    check_test(&resource_content(
        true,
        &["function"],
        "exception_and_type.mamba",
    ))
}

#[test]
fn allowed_exception() -> CheckTestRet {
    check_test(&resource_content(
        true,
        &["function"],
        "allowed_exception.mamba",
    ))
}

#[test]
fn call_mut_function() -> CheckTestRet {
    check_test(&resource_content(
        true,
        &["function"],
        "call_mut_function.mamba",
    ))
}

#[test]
fn allowed_pass() -> CheckTestRet {
    check_test(&resource_content(true, &["function"], "allowed_pass.mamba"))
}
