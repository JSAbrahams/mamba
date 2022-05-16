use mamba::generate::gen;
use mamba::parse::parse;

use crate::common::*;

#[test]
fn core_for_statements() {
    let source = resource_content(true, &["control_flow"], "for_statements.mamba");
    to_py!(source);
}

#[test]
fn core_if() {
    let source = resource_content(true, &["control_flow"], "if.mamba");
    to_py!(source);
}

#[test]
#[ignore]
fn core_match_statements() {
    let source = resource_content(true, &["control_flow"], "match.mamba");
    to_py!(source);
}

#[test]
fn core_while_statements() {
    let source = resource_content(true, &["control_flow"], "while.mamba");
    to_py!(source);
}
