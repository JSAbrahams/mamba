use mamba::check::check_all;
use mamba::generate::gen;
use mamba::parse::ast::AST;

use crate::common::*;

#[test]
fn for_statements() {
    to_py!(resource_content(
        true,
        &["control_flow"],
        "for_statements.mamba"
    ));
}

#[test]
fn if_stmt() {
    to_py!(resource_content(true, &["control_flow"], "if.mamba"));
}

#[test]
fn match_stmt() {
    to_py!(resource_content(
        true,
        &["control_flow"],
        "match_stmt.mamba"
    ));
}

#[test]
fn while_stmt() {
    to_py!(resource_content(true, &["control_flow"], "while.mamba"));
}
