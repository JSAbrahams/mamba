use mamba::Arguments;

use crate::system::{OutTestRet, test_directory, test_directory_args};

#[test]
fn handle() -> OutTestRet {
    test_directory_args(true, &["error"], &["error", "target"], "handle", &args)
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
