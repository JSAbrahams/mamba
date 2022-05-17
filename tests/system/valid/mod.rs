use crate::system::{OutTestRet, test_directory};

pub mod access;
pub mod class;
pub mod collection;
pub mod control_flow;
pub mod definition;
pub mod error;
pub mod function;
pub mod operation;

#[test]
fn empty_file() -> OutTestRet {
    test_directory(true, &[], &["target"], "empty_file")
}

#[test]
fn doc() -> OutTestRet {
    test_directory(true, &[], &["target"], "doc")
}
