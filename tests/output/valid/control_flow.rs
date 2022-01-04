use crate::output::test_directory;

#[test]
fn for_ast_verify() -> Result<(), Vec<String>> {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "for_statements")
}

#[test]
fn for_over_collection_of_tuple() -> Result<(), Vec<String>> {
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
#[ignore] // Need to handle tuple identifiers
fn match_ast_verify() -> Result<(), Vec<String>> {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "match")
}
