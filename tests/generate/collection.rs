use mamba::check::check_all;
use mamba::generate::gen;
use mamba::parse::parse;

use crate::common::*;

#[test]
fn core_list() {
    let source = resource_content(true, &["collection"], "tuple.mamba");
    to_py!(source);
}

#[test]
fn core_set() {
    let source = resource_content(true, &["collection"], "set.mamba");
    to_py!(source);
}

#[test]
fn core_tuple() {
    let source = resource_content(true, &["collection"], "tuple.mamba");
    to_py!(source);
}
