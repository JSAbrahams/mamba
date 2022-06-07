use crate::system::{OutTestRet, test_directory};

#[test]
fn arithmetic() -> OutTestRet {
    test_directory(true, &["operation"], &["operation", "target"], "arithmetic")
}

#[test]
fn assign_types() -> OutTestRet {
    test_directory(true, &["operation"], &["operation", "target"], "assign_types")
}

#[test]
#[ignore] // See if we can modify type checker so it takes largest common denominator of type
fn assign_types_no_annotation() -> OutTestRet {
    test_directory(true, &["operation"], &["operation", "target"], "assign_types_no_annotation")
}

#[test]
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

#[test]
#[ignore] // investigate whether this should in fact, pass
fn type_alias_primitive() -> OutTestRet {
    test_directory(true, &["operation"], &["operation", "target"], "type_alias_primitive")
}
