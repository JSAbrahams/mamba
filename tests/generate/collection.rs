use mamba::check::check_all;
use mamba::generate::gen;
use mamba::parse::ast::AST;

use crate::common::*;

#[test]
fn core_list() {
    let source = resource_content(true, &["collection"], "tuple.mamba");
    let _ = to_py!(source);
}

#[test]
fn core_set() {
    let source = resource_content(true, &["collection"], "set.mamba");
    let _ = to_py!(source);
}

#[test]
fn core_tuple() {
    let source = resource_content(true, &["collection"], "tuple.mamba");
    let _ = to_py!(source);
}
