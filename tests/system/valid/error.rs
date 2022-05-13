use crate::system::{test_directory, OutTestRet};

#[test]
fn handle_ast_verify() -> OutTestRet {
    test_directory(true, &["error"], &["error", "target"], "handle")
}

#[test]
fn exception_ast_verify() -> OutTestRet {
    test_directory(true, &["error"], &["error", "target"], "exception")
}

#[test]
fn raise_ast_verify() -> OutTestRet {
    test_directory(true, &["error"], &["error", "target"], "raise")
}

#[test]
fn with_ast_verify() -> OutTestRet {
    test_directory(true, &["error"], &["error", "target"], "with")
}
