use crate::util::*;
use mamba::lexer::tokenize;
use mamba::parser::parse;

#[test]
fn parse_for_statements() {
    let source = valid_resource_content(&["control_flow"],"for_statements.mamba");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_if() {
    let source = valid_resource_content(&["control_flow"],"if.mamba");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_match_statements() {
    let source = valid_resource_content(&["control_flow"],"match_statements.mamba");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_while_statements() {
    let source = valid_resource_content(&["control_flow"],"while_statements.mamba");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}
