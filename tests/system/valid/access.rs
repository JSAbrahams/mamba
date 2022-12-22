use crate::system::{OutTestRet, test_directory};

#[test]
#[ignore] // #383
fn simple_index() -> OutTestRet {
    test_directory(true, &["access"], &["access", "target"], "simple_index")
}

#[test]
#[ignore] // Dictionaries not implemented yet
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
#[ignore] // #383
fn access_string() -> OutTestRet {
    test_directory(true, &["access"], &["access", "target"], "access_string")
}
