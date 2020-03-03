use crate::common::resource_content;
use mamba::lex::tokenize;
use mamba::parse::parse;

#[test]
fn assigns_and_while() {
    let source = resource_content(false, &["syntax"], "assign_and_while.mamba");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}
