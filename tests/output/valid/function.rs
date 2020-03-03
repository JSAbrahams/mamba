use crate::output::test_directory;

#[test]
fn call_ast_verify() -> Result<(), Vec<String>> {
    test_directory(true, &["function"], &["function", "target"], "calls")
}

#[test]
fn definition_ast_verify() -> Result<(), Vec<String>> {
    test_directory(true, &["function"], &["function", "target"], "definition")
}

#[test]
fn function_with_defaults_ast_verify() -> Result<(), Vec<String>> {
    test_directory(true, &["function"], &["function", "target"], "function_with_defaults")
}
