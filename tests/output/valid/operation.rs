use crate::output::{OutTestRet, test_directory};

#[test]
fn arithmetic_ast_verify() -> OutTestRet {
    test_directory(true, &["operation"], &["operation", "target"], "arithmetic")
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
