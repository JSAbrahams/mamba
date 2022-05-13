use crate::system::{test_directory, OutTestRet};

#[test]
#[ignore] // No list builder construct yet
fn list_verify() -> OutTestRet {
    test_directory(true, &["collection"], &["collection", "target"], "list")
}

#[test]
#[ignore] // No set/map builder construct yet
fn map_verify() -> OutTestRet {
    test_directory(true, &["collection"], &["collection", "target"], "map")
}

#[test]
#[ignore] // No set builder construct yet
fn set_verify() -> OutTestRet {
    test_directory(true, &["collection"], &["collection", "target"], "set")
}

#[test]
#[ignore] // Type checker cannot handle assigning to tuples
fn tuple_verify() -> OutTestRet {
    test_directory(true, &["collection"], &["collection", "target"], "tuple")
}
