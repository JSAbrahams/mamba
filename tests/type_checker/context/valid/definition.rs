use crate::common::resource_content;
use mamba::lexer::tokenize;
use mamba::parser::parse;
use mamba::type_checker::check_all;

#[test]
fn nested_mut_field() {
    let source = resource_content(true, &["definition"], "nested_mut_field.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap();
}
