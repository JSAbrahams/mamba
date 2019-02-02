use my_lang::desugarer::desugar;
use my_lang::lexer::tokenize;
use my_lang::parser::parse;
use my_lang::type_checker::type_check;
use crate::util::valid_resource;

mod util;

#[test]
#[ignore]
fn interpret_assigns_and_while() {
    let source = valid_resource("assign_and_while.txt");

    let tokens = tokenize(source).unwrap();
    let ast_nodes = parse(tokens).unwrap();
    let type_checked = type_check(ast_nodes);
    let desugared = desugar(type_checked);
}
