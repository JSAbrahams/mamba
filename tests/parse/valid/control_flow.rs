use mamba::lex::tokenize;
use mamba::parse::parse;

use crate::common::resource_content;

#[test]
fn while_statements() {
    let source = resource_content(true, &["control_flow"], "while.mamba");
    parse(&tokenize(&source).unwrap()).unwrap();
}
