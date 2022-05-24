use mamba::check::check_all;
use mamba::generate::gen;
use mamba::parse::parse;

use crate::common::*;

#[test]
fn core_for_statements() {
    to_py!(resource_content(true, &["control_flow"], "for_statements.mamba"));
}

#[test]
fn core_if() {
    to_py!(resource_content(true, &["control_flow"], "if.mamba"));
}

#[test]
fn core_match_statements() {
    to_py!(resource_content(true, &["control_flow"], "match_stmt.mamba"));
}

#[test]
fn core_while_statements() {
    to_py!(resource_content(true, &["control_flow"], "while.mamba"));
}
