use crate::output::test_directory;

#[test]
#[ignore]
fn generics_ast_verify() -> Result<(), Vec<String>> {
    test_directory(true, &["class"], &["class", "target"], "generics")
}

#[test]
fn import_ast_verify() -> Result<(), Vec<String>> {
    test_directory(true, &["class"], &["class", "target"], "import")
}

#[test]
fn doc_strings_ast_verify() -> Result<(), Vec<String>> {
    test_directory(true, &["class"], &["class", "target"], "doc_strings")
}

#[test]
fn parent_ast_verify() -> Result<(), Vec<String>> {
    test_directory(true, &["class"], &["class", "target"], "parent")
}

#[test]
#[ignore]
fn types_ast_verify() -> Result<(), Vec<String>> {
    test_directory(true, &["class"], &["class", "target"], "types")
}
