use crate::util::valid_resource;

use my_lang::lexer::tokenize;
use my_lang::parser::parse;
use my_lang::desugarer::desugar;
use my_lang::core::to_py_source;

mod util;

#[test]
fn class_to_python() {
    let source = valid_resource("assign_and_while.txt");

    let tokens = tokenize(source).unwrap();
    let ast_tree = parse(tokens).unwrap();
    let core_tree = desugar(ast_tree);
    to_py_source(core_tree);
}
