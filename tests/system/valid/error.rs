use crate::system::{OutTestRet, test_directory};

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
#[ignore] // Type aliases
fn with_ast_verify() -> OutTestRet {
    test_directory(true, &["error"], &["error", "target"], "with")
}
