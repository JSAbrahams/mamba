use crate::system::{OutTestRet, test_directory};

#[test]
fn long_f_string() -> OutTestRet {
    test_directory(true, &["definition"], &["definition", "target"], "long_f_string")
}

#[test]
fn assign_tuples() -> OutTestRet {
    test_directory(true, &["definition"], &["definition", "target"], "assign_tuples")
}

#[test]
fn function_ret_super_in_class() -> OutTestRet {
    test_directory(true, &["definition"], &["definition", "target"], "function_ret_super_in_class")
}

#[test]
fn function_with_if() -> OutTestRet {
    test_directory(true, &["definition"], &["definition", "target"], "function_with_if")
}

#[test]
fn function_with_if_and_raise() -> OutTestRet {
    test_directory(true, &["definition"], &["definition", "target"], "function_with_if_and_raise")
}

#[test]
fn function_with_nested_if() -> OutTestRet {
    test_directory(true, &["definition"], &["definition", "target"], "function_with_nested_if")
}

#[test]
fn function_with_match() -> OutTestRet {
    test_directory(true, &["definition"], &["definition", "target"], "function_with_match")
}

#[test]
fn function_with_try_except() -> OutTestRet {
    test_directory(true, &["definition"], &["definition", "target"], "function_with_try_except")
}

#[test]
fn ternary() -> OutTestRet {
    test_directory(true, &["definition"], &["definition", "target"], "ternary")
}

#[test]
fn function_ret_super() -> OutTestRet {
    test_directory(true, &["definition"], &["definition", "target"], "function_ret_super")
}
