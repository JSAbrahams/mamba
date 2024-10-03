use crate::system::{test_directory, OutTestRet};

#[test]
fn call_with_class_child() -> OutTestRet {
    test_directory(
        true,
        &["call"],
        &["call", "target"],
        "call_with_class_child",
    )
}

#[test]
fn input() -> OutTestRet {
    test_directory(true, &["call"], &["call", "target"], "input")
}
