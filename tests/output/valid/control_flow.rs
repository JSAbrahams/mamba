use crate::output::test_directory;

#[test]
fn for_ast_verify() -> Result<(), Vec<String>> {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "for_statements")
}

#[test]
fn for_over_collection_of_tuple() -> Result<(), Vec<String>> {
    loggerv::Logger::new().verbosity(3).module_path(false).init().unwrap();
    test_directory(true, &["control_flow"], &["control_flow", "target"], "for_over_collection_of_tuple")
}

#[test]
fn if_ast_verify() -> Result<(), Vec<String>> {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "if")
}

#[test]
fn while_ast_verify() -> Result<(), Vec<String>> {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "while")
}

#[test]
fn match_ast_verify() -> Result<(), Vec<String>> {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "match")
}
