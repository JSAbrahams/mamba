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
fn greater_than_int() -> OutTestRet {
    test_directory(true, &["operation"], &["operation", "target"], "greater_than_int")
}

#[test]
fn greater_than_other_int() -> OutTestRet {
    test_directory(true, &["operation"], &["operation", "target"], "greater_than_other_int")
}

#[test]
fn in_set_is_bool() -> OutTestRet {
    test_directory(true, &["operation"], &["operation", "target"], "in_set_is_bool")
}

#[test]
fn multiply_other_int() -> OutTestRet {
    test_directory(true, &["operation"], &["operation", "target"], "multiply_other_int")
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
fn equality_different_types() -> OutTestRet {
    test_directory(true, &["operation"], &["operation", "target"], "equality_different_types")
}

#[test]
#[ignore] // investigate whether this should in fact, pass
fn type_alias_primitive() -> OutTestRet {
    test_directory(true, &["operation"], &["operation", "target"], "type_alias_primitive")
}
