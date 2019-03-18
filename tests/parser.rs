use crate::util::valid_resource_contents;
use mamba::lexer::tokenize;
use mamba::parser::parse;

mod util;

#[test]
fn parse_assigns_and_while() {
    let source = valid_resource_contents("assign_and_while.txt");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_class() {
    let source = valid_resource_contents("class.txt");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_collections() {
    let source = valid_resource_contents("collections.txt");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_empty_file() {
    let source = valid_resource_contents("empty_file.txt");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_for_statements() {
    let source = valid_resource_contents("for_statements.txt");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_if() {
    let source = valid_resource_contents("if.txt");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_tuples() {
    let source = valid_resource_contents("tuples.txt");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_when_statements() {
    let source = valid_resource_contents("when_statements.txt");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_while_statements() {
    let source = valid_resource_contents("while_statements.txt");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_function_definitions() {
    let source = valid_resource_contents("function_definitions.txt");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_function_calling() {
    let source = valid_resource_contents("function_calling.txt");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_infix_function_calling() {
    let source = valid_resource_contents("infix_function_calling.txt");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}
