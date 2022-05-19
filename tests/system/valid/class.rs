use crate::system::{OutTestRet, test_directory};

#[test]
#[ignore]
fn generics_ast_verify() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "generics")
}

#[test]
fn assign_types_nested() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "assign_types_nested")
}

#[test]
#[ignore] // Should be fixed within this PR ideally
fn assign_types_double_nested() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "assign_types_double_nested")
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
fn top_level_tuple() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "top_level_tuple")
}

#[test]
fn tuple_as_class_verify() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "tuple_as_class")
}
