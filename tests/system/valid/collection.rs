use mamba::Arguments;

use crate::system::{OutTestRet, test_directory, test_directory_args};

#[test]
fn infer_collection_type() -> OutTestRet {
    test_directory(true, &["collection"], &["collection", "target"], "infer_collection_type")
}

#[test]
fn infer_collection_type_for_fun() -> OutTestRet {
    test_directory(true, &["collection"], &["collection", "target"], "infer_collection_type_for_fun")
}

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
fn tuple_verify() -> OutTestRet {
    let args = Arguments { annotate: false }; // Type annotations in output wrong
    test_directory_args(true, &["collection"], &["collection", "target"], "tuple", &args)
}
