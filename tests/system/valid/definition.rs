use crate::system::{OutTestRet, test_directory};

#[test]
fn long_f_string() -> OutTestRet {
    test_directory(true, &["definition"], &["definition", "target"], "long_f_string")
}

#[test]
fn assign_tuples() -> OutTestRet {
    test_directory(true, &["definition"], &["definition", "target"], "assign_tuples")
}
