use crate::system::{OutTestRet, test_directory};

#[test]
fn arithmetic_ast_verify() -> OutTestRet {
    test_directory(true, &["operation"], &["operation", "target"], "arithmetic")
}

#[test]
#[ignore] // See if we can modify type checker so it takes largest common denominator of type
fn assign_types() -> OutTestRet {
    test_directory(true, &["operation"], &["operation", "target"], "assign_types")
}

#[test]
#[ignore] // a is undefined in reassign for some reason
fn assign_types_nested() -> OutTestRet {
    test_directory(true, &["operation"], &["operation", "target"], "assign_types_nested")
}

#[test]
fn primitives_ast_verify() -> OutTestRet {
    test_directory(true, &["operation"], &["operation", "target"], "primitives")
}

#[test]
fn bitwise_ast_verify() -> OutTestRet {
    test_directory(true, &["operation"], &["operation", "target"], "bitwise")
}

#[test]
fn boolean_ast_verify() -> OutTestRet {
    test_directory(true, &["operation"], &["operation", "target"], "boolean")
}
