use crate::util::*;
use mamba::core::to_py_source;
use mamba::desugarer::desugar;
use mamba::lexer::tokenize;
use mamba::parser::parse;

#[test]
fn core_class() {
    let source = valid_resource_content(&["class"], "class.mamba");
    to_py!(source);
}
