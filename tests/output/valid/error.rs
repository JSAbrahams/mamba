use crate::output::test_directory;

#[test]
fn handle_ast_verify() -> Result<(), Vec<String>> {
    test_directory(true, &["error"], &["error", "target"], "handle")
}

#[test]
fn exception_ast_verify() -> Result<(), Vec<String>> {
    test_directory(true, &["error"], &["error", "target"], "exception")
}

#[test]
fn raise_ast_verify() -> Result<(), Vec<String>> {
    test_directory(true, &["error"], &["error", "target"], "raise")
}

#[test]
fn with_ast_verify() -> Result<(), Vec<String>> {
    test_directory(true, &["error"], &["error", "target"], "with")
}
