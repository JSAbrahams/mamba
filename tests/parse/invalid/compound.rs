use mamba::parse::lex::tokenize;
use mamba::parse::parse;

use crate::common::resource_content;

#[test]
fn assigns_and_while() {
    let source = resource_content(false, &["syntax"], "assign_and_while.mamba");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}
