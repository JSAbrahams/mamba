use crate::output::test_directory;

#[test]
fn arithmetic_ast_verify() -> Result<(), Vec<String>> {
    test_directory(true, &["operation"], &["operation", "target"], "arithmetic")
}

#[test]
fn primitives_ast_verify() -> Result<(), Vec<String>> {
    test_directory(true, &["operation"], &["operation", "target"], "primitives")
}

#[test]
fn bitwise_ast_verify() -> Result<(), Vec<String>> {
    test_directory(true, &["operation"], &["operation", "target"], "bitwise")
}

#[test]
fn boolean_ast_verify() -> Result<(), Vec<String>> {
    test_directory(true, &["operation"], &["operation", "target"], "boolean")
}
