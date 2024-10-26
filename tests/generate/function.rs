use mamba::check::check_all;
use mamba::generate::gen;
use mamba::parse::ast::AST;

use crate::common::*;

#[test]
fn function_definitions() {
    let _ = to_py!(resource_content(true, &["function"], "definition.mamba"));
}

#[test]
fn function_calling() {
    let _ = to_py!(resource_content(true, &["function"], "calls.mamba"));
}
