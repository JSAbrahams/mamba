use crate::system::{OutTestRet, test_directory};

#[test]
fn simple_index() -> OutTestRet {
    test_directory(true, &["access"], &["access", "target"], "simple_index")
}

#[test]
fn dictionary_access() -> OutTestRet {
    test_directory(true, &["access"], &["access", "target"], "dictionary_access")
}

#[test]
fn simple_list_access() -> OutTestRet {
    test_directory(true, &["access"], &["access", "target"], "simple_list_access")
}

#[test]
fn index_via_function() -> OutTestRet {
    test_directory(true, &["access"], &["access", "target"], "index_via_function")
}

#[test]
fn access_string() -> OutTestRet {
    test_directory(true, &["access"], &["access", "target"], "access_string")
}
