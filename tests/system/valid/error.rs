use crate::system::{OutTestRet, test_directory};

#[test]
fn handle() -> OutTestRet {
    test_directory(true, &["error"], &["error", "target"], "handle")
}

#[test]
fn exception() -> OutTestRet {
    test_directory(true, &["error"], &["error", "target"], "exception")
}

#[test]
fn exception_in_fun() -> OutTestRet {
    test_directory(true, &["error"], &["error", "target"], "exception_in_fun")
}

#[test]
fn handle_var_usable_after() -> OutTestRet {
    test_directory(true, &["error"], &["error", "target"], "handle_var_usable_after")
}

#[test]
fn exception_in_fun_super() -> OutTestRet {
    test_directory(true, &["error"], &["error", "target"], "exception_in_fun_super")
}

#[test]
fn nested_exception() -> OutTestRet {
    test_directory(true, &["error"], &["error", "target"], "nested_exception")
}

#[test]
fn raise() -> OutTestRet {
    test_directory(true, &["error"], &["error", "target"], "raise")
}

#[test]
fn with() -> OutTestRet {
    test_directory(true, &["error"], &["error", "target"], "with")
}
