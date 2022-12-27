use crate::system::{OutTestRet, test_directory};

#[test]
fn call_with_class_child() -> OutTestRet {
    test_directory(true, &["call"], &["call", "target"], "call_with_class_child")
}
