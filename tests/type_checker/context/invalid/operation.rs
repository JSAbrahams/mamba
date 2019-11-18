use crate::common::resource_content;
use mamba::lexer::tokenize;
use mamba::parser::parse;
use mamba::type_checker::check_all;

#[test]
fn string_minus() {
    let source = resource_content(false, &["type", "operation"], "string_minus.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}
