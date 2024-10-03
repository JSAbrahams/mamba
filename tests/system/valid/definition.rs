use crate::system::{test_directory, OutTestRet};

#[test]
fn long_f_string() -> OutTestRet {
    test_directory(
        true,
        &["definition"],
        &["definition", "target"],
        "long_f_string",
    )
}

#[test]
fn assign_tuples() -> OutTestRet {
    test_directory(
        true,
        &["definition"],
        &["definition", "target"],
        "assign_tuples",
    )
}

#[test]
fn assign_to_nullable_in_function() -> OutTestRet {
    test_directory(
        true,
        &["definition"],
        &["definition", "target"],
        "assign_to_nullable_in_function",
    )
}

#[test]
fn assign_with_if() -> OutTestRet {
    test_directory(
        true,
        &["definition"],
        &["definition", "target"],
        "assign_with_if",
    )
}

#[test]
#[ignore] // annotating output reveals bug in check stage
fn assign_with_if_different_types() -> OutTestRet {
    test_directory(
        true,
        &["definition"],
        &["definition", "target"],
        "assign_with_if_different_types",
    )
}

#[test]
fn assign_with_match() -> OutTestRet {
    test_directory(
        true,
        &["definition"],
        &["definition", "target"],
        "assign_with_match",
    )
}

#[test]
fn assign_with_match_different_types() -> OutTestRet {
    test_directory(
        true,
        &["definition"],
        &["definition", "target"],
        "assign_with_match_different_types",
    )
}

#[test]
fn assign_with_match_type_annotation() -> OutTestRet {
    test_directory(
        true,
        &["definition"],
        &["definition", "target"],
        "assign_with_match_type_annotation",
    )
}

#[test]
fn assign_with_nested_if() -> OutTestRet {
    test_directory(
        true,
        &["definition"],
        &["definition", "target"],
        "assign_with_nested_if",
    )
}

#[test]
fn assign_with_try_except() -> OutTestRet {
    test_directory(
        true,
        &["definition"],
        &["definition", "target"],
        "assign_with_try_except",
    )
}

#[test]
fn function_ret_super_in_class() -> OutTestRet {
    test_directory(
        true,
        &["definition"],
        &["definition", "target"],
        "function_ret_super_in_class",
    )
}

#[test]
fn function_no_return_type() -> OutTestRet {
    test_directory(
        true,
        &["definition"],
        &["definition", "target"],
        "function_no_return_type",
    )
}

#[test]
fn function_with_if() -> OutTestRet {
    test_directory(
        true,
        &["definition"],
        &["definition", "target"],
        "function_with_if",
    )
}

#[test]
fn function_with_if_and_raise() -> OutTestRet {
    test_directory(
        true,
        &["definition"],
        &["definition", "target"],
        "function_with_if_and_raise",
    )
}

#[test]
fn function_with_nested_if() -> OutTestRet {
    test_directory(
        true,
        &["definition"],
        &["definition", "target"],
        "function_with_nested_if",
    )
}

#[test]
fn function_with_match() -> OutTestRet {
    test_directory(
        true,
        &["definition"],
        &["definition", "target"],
        "function_with_match",
    )
}

#[test]
fn ternary() -> OutTestRet {
    test_directory(true, &["definition"], &["definition", "target"], "ternary")
}

#[test]
fn function_ret_super() -> OutTestRet {
    test_directory(
        true,
        &["definition"],
        &["definition", "target"],
        "function_ret_super",
    )
}

#[test]
fn tuple_modify_mut() -> OutTestRet {
    test_directory(
        true,
        &["definition"],
        &["definition", "target"],
        "tuple_modify_mut",
    )
}

#[test]
fn tuple_non_lit_modify_mut() -> OutTestRet {
    test_directory(
        true,
        &["definition"],
        &["definition", "target"],
        "tuple_non_lit_modify_mut",
    )
}
