use crate::system::{test_directory, OutTestRet};

pub mod access;
pub mod call;
pub mod class;
pub mod collection;
pub mod control_flow;
pub mod definition;
pub mod error;
pub mod function;
pub mod operation;
pub mod readme_examples;

#[test]
fn empty_file() -> OutTestRet {
    test_directory(true, &[], &["target"], "empty_file")
}

#[test]
fn doc() -> OutTestRet {
    test_directory(true, &[], &["target"], "doc")
}

#[test]
fn std_functions() -> OutTestRet {
    test_directory(true, &[], &["target"], "std_functions")
}
