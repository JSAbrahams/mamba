use mamba::check::check_all;
use mamba::generate::gen;
use mamba::parse::ast::AST;

use crate::common::*;

#[test]
fn function_definitions() {
    to_py!(resource_content(true, &["function"], "definition.mamba"));
}

#[test]
fn function_calling() {
    to_py!(resource_content(true, &["function"], "calls.mamba"));
}
