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
#[ignore] // See #367
fn if_in_if_cond() -> OutTestRet {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "if_in_if_cond")
}

#[test]
#[ignore] // See #367
fn if_in_for_loop() -> OutTestRet {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "if_in_for_loop")
}

#[test]
#[ignore] // not sure if the check stage should pass as of yet
fn if_two_types() -> OutTestRet {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "if_two_types")
}

#[test]
fn while_ast_verify() -> OutTestRet {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "while")
}


#[test]
fn match_stmt_ast_verify() -> OutTestRet {
    test_directory(true, &["control_flow"], &["control_flow", "target"], "match_stmt")
}
