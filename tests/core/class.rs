use crate::common::*;
use mamba::core::to_py_source;
use mamba::desugar::desugar;
use mamba::lexer::tokenize;
use mamba::parser::parse;

#[test]
fn core_class() {
    let source = valid_resource_content(&["class"], "class.mamba");
    to_py!(source);
}

#[test]
fn core_imports() {
    let source = valid_resource_content(&["class"], "import.mamba");
    to_py!(source);
}
