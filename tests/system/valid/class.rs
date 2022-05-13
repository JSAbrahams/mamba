use crate::system::{test_directory, OutTestRet};

#[test]
#[ignore]
fn generics_ast_verify() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "generics")
}

#[test]
fn import_ast_verify() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "import")
}

#[test]
fn doc_strings_ast_verify() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "doc_strings")
}

#[test]
fn parent_ast_verify() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "parent")
}

#[test]
fn types_ast_verify() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "types")
}

#[test]
fn tuple_as_class_verify() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "tuple_as_class")
}
