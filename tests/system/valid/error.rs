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
fn raise() -> OutTestRet {
    test_directory(true, &["error"], &["error", "target"], "raise")
}

#[test]
fn with() -> OutTestRet {
    test_directory(true, &["error"], &["error", "target"], "with")
}
