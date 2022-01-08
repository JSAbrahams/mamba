use mamba::lex::tokenize;
use mamba::parse::parse;

use crate::common::resource_content;

#[test]
fn for_statements() {
    let source = resource_content(true, &["control_flow"], "for_statements.mamba");
    parse(&tokenize(&source).unwrap()).unwrap();
}

#[test]
fn if_stmt() {
    let source = resource_content(true, &["control_flow"], "if.mamba");
    assert!(parse(&tokenize(&source).unwrap()).is_ok());
}

#[test]
fn match_statements() {
    let source = resource_content(true, &["control_flow"], "match.mamba");
    parse(&tokenize(&source).unwrap()).unwrap();
}
