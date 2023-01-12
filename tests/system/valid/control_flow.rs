use crate::system::{OutTestRet, test_directory};

#[test]
fn assign_if() -> OutTestRet {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "assign_if")
}

#[test]
fn assign_match() -> OutTestRet {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "assign_match")
}

#[test]
fn double_assign_if() -> OutTestRet {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "double_assign_if")
}

#[test]
fn for_statements() -> OutTestRet {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "for_statements")
}

#[test]
fn handle_in_if() -> OutTestRet {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "handle_in_if")
}

#[test]
fn for_over_collection_of_tuple() -> OutTestRet {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "for_over_collection_of_tuple")
}

#[test]
#[ignore] // trips up when comparing range_iterator and str_iterator
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
fn if_in_if_cond() -> OutTestRet {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "if_in_if_cond")
}

#[test]
#[ignore] // See #367
fn if_in_for_loop() -> OutTestRet {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "if_in_for_loop")
}

#[test]
fn if_two_types() -> OutTestRet {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "if_two_types")
}

#[test]
fn match_dont_remove_shadowed() -> OutTestRet {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "match_dont_remove_shadowed")
}

#[test]
fn matches_in_if() -> OutTestRet {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "matches_in_if")
}

#[test]
fn shadow_in_if_arms() -> OutTestRet {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "shadow_in_if_arms")
}

#[test]
fn shadow_in_if_arms_then() -> OutTestRet {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "shadow_in_if_arms_then")
}

#[test]
fn shadow_in_if_arms_else() -> OutTestRet {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "shadow_in_if_arms_else")
}

#[test]
fn while_ast_verify() -> OutTestRet {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "while")
}

#[test]
fn match_stmt_ast_verify() -> OutTestRet {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "match_stmt")
}
