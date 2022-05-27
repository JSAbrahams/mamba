use crate::system::{OutTestRet, test_directory};

#[test]
#[ignore] // See #320
fn generics() -> OutTestRet {
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
fn doc_strings() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "doc_strings")
}

#[test]
#[ignore] // See #314, #315
fn multiple_parent() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "multiple_parent")
}

#[test]
fn parent() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "parent")
}

#[test]
fn types() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "types")
}

#[test]
fn top_level_tuple() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "top_level_tuple")
}

#[test]
fn top_level_unassigned_but_nullable() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "top_level_unassigned_but_nullable")
}

#[test]
fn tuple_as_class() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "tuple_as_class")
}

#[test]
fn unassigned_tuple_second_nullable() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "unassigned_tuple_second_nullable")
}
