use crate::system::{test_directory, OutTestRet};

#[test]
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
