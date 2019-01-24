use crate::util::valid_resource;
use my_lang::lexer::tokenize;
use my_lang::parser::parse;

mod util;

#[test]
fn parse_assigns_and_while() {
    let source = valid_resource("assign_and_while.txt");
    assert_ok!(parse(tokenize(source).unwrap()));
}

#[test]
fn parse_class() {
    let source = valid_resource("class.txt");
    assert_ok!(parse(tokenize(source).unwrap()));
}

#[test]
fn parse_empty_file() {
    let source = valid_resource("empty_file.txt");
    assert_ok!(parse(tokenize(source).unwrap()));
}

#[test]
fn parse_for_statements() {
    let source = valid_resource("for_statements.txt");
    assert_ok!(parse(tokenize(source).unwrap()));
}

#[test]
fn parse_if() {
    let source = valid_resource("if.txt");
    assert_ok!(parse(tokenize(source).unwrap()));
}

#[test]
fn parse_tuples() {
    let source = valid_resource("tuples.txt");
    assert_ok!(parse(tokenize(source).unwrap()));
}

#[test]
fn parse_when_statements() {
    let source = valid_resource("when_statements.txt");
    assert_ok!(parse(tokenize(source).unwrap()));
}

#[test]
fn parse_while_statements() {
    let source = valid_resource("while_statements.txt");
    assert_ok!(parse(tokenize(source).unwrap()));
}

#[test]
fn parse_function_definitions() {
    let source = valid_resource("function_definitions.txt");
    assert_ok!(parse(tokenize(source).unwrap()));
}

#[test]
fn parse_function_calling() {
    let source = valid_resource("function_calling.txt");
    assert_ok!(parse(tokenize(source).unwrap()))
}
