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
fn function_ret_super() -> OutTestRet {
    test_directory(true, &["definition"], &["definition", "target"], "function_ret_super")
}
