use crate::system::{OutTestRet, test_directory};

#[test]
fn assign_to_nullable_field() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "assign_to_nullable_field")
}

#[test]
fn generics() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "generics")
}

#[test]
fn assign_types_nested() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "assign_types_nested")
}

#[test]
fn assign_with_generic() -> OutTestRet {
    loggerv::Logger::new().module_path(false).verbosity(3).init().unwrap();
    test_directory(true, &["class"], &["class", "target"], "assign_with_generic")
}

#[test]
fn assign_with_generic_two_generics() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "assign_with_generic_two_generics")
}

#[test]
fn class_super_one_line_init() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "class_super_one_line_init")
}

#[test]
fn assign_types_double_nested() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "assign_types_double_nested")
}

#[test]
fn print_types_double_nested() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "print_types_double_nested")
}

#[test]
fn import_ast_verify() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "import")
}

#[test]
fn generic_unknown_type_unused() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "generic_unknown_type_unused")
}

#[test]
fn same_var_different_type() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "same_var_different_type")
}

#[test]
fn doc_strings() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "doc_strings")
}

#[test]
fn fun_with_body_in_interface() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "fun_with_body_in_interface")
}

#[test]
fn multiple_parent() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "multiple_parent")
}

#[test]
fn shadow() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "shadow")
}

#[test]
fn parent() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "parent")
}

#[test]
fn types() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "types")
}

#[test]
fn top_level_tuple() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "top_level_tuple")
}

#[test]
fn top_level_unassigned_but_nullable() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "top_level_unassigned_but_nullable")
}

#[test]
fn tuple_as_class() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "tuple_as_class")
}

#[test]
fn unassigned_tuple_second_nullable() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "unassigned_tuple_second_nullable")
}

#[test]
fn var_from_outside_class() -> OutTestRet {
    test_directory(true, &["class"], &["class", "target"], "var_from_outside_class")
}
