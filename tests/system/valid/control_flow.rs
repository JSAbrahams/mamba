use crate::system::{OutTestRet, test_directory};

#[test]
fn for_statements() -> OutTestRet {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "for_statements")
}

#[test]
fn for_over_collection_of_tuple() -> OutTestRet {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "for_over_collection_of_tuple")
}

#[test]
fn for_over_type_union() -> OutTestRet {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "for_over_type_union")
}

#[test]
fn for_over_range_from_func() -> OutTestRet {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "for_over_range_from_func")
}

#[test]
fn if_ast_verify() -> OutTestRet {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "if")
}

#[test]
fn while_ast_verify() -> OutTestRet {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "while")
}

#[test]
fn match_stmt_ast_verify() -> OutTestRet {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "match_stmt")
}
