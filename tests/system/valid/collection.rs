use mamba::Arguments;

use crate::system::{test_directory, test_directory_args, OutTestRet};

#[test]
fn collection_type() -> OutTestRet {
    test_directory(
        true,
        &["collection"],
        &["collection", "target"],
        "collection_type",
    )
}

#[test]
fn infer_collection_type() -> OutTestRet {
    test_directory(
        true,
        &["collection"],
        &["collection", "target"],
        "infer_collection_type",
    )
}

#[test]
fn infer_collection_type_for_fun() -> OutTestRet {
    test_directory(
        true,
        &["collection"],
        &["collection", "target"],
        "infer_collection_type_for_fun",
    )
}

#[test]
fn list_verify() -> OutTestRet {
    test_directory(true, &["collection"], &["collection", "target"], "list")
}

#[test]
fn dictionary_verify() -> OutTestRet {
    test_directory(
        true,
        &["collection"],
        &["collection", "target"],
        "dictionary",
    )
}

#[test]
fn dictionary_builder_verify() -> OutTestRet {
    test_directory(
        true,
        &["collection"],
        &["collection", "target"],
        "dictionary_builder",
    )
}

#[test]
fn dictionary_in_fun() -> OutTestRet {
    test_directory(
        true,
        &["collection"],
        &["collection", "target"],
        "dictionary_in_fun",
    )
}

#[test]
fn nested_list_builder() -> OutTestRet {
    test_directory(
        true,
        &["collection"],
        &["collection", "target"],
        "nested_list_builder",
    )
}

#[test]
fn nested_set_builder() -> OutTestRet {
    test_directory(
        true,
        &["collection"],
        &["collection", "target"],
        "nested_set_builder",
    )
}

#[test]
fn set_verify() -> OutTestRet {
    test_directory(true, &["collection"], &["collection", "target"], "set")
}

#[test]
fn set_union() -> OutTestRet {
    test_directory(
        true,
        &["collection"],
        &["collection", "target"],
        "set_union",
    )
}

#[test]
fn tuple_verify() -> OutTestRet {
    let args = Arguments { annotate: false }; // Type annotations in output wrong
    test_directory_args(
        true,
        &["collection"],
        &["collection", "target"],
        "tuple",
        &args,
    )
}
