use crate::common::*;
use mamba::core::to_py_source;
use mamba::desugar::desugar;
use mamba::lexer::tokenize;
use mamba::parser::parse;

#[test]
fn core_class() {
    let source = resource_content(true, &["class"], "class.mamba");
    to_py!(source);
}

#[test]
fn core_imports() {
    let source = resource_content(true, &["class"], "import.mamba");
    to_py!(source);
}
