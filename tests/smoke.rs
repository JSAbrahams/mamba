use crate::util::valid_resource_contents;
use mamba::core::to_py_source;
use mamba::desugarer::desugar;
use mamba::lexer::tokenize;
use mamba::parser::parse;

mod util;

#[test]
fn class_to_python() {
    let source = valid_resource_contents("class.txt");

    let tokens = tokenize(&source).unwrap();
    let ast_tree = parse(&tokens).unwrap();
    let core_tree = desugar(&ast_tree);
    println!("{}", to_py_source(&core_tree));
}
