use crate::util::valid_resource_content;
use mamba::lexer::tokenize;
use mamba::parser::parse;

mod util;

#[test]
fn parse_assigns_and_while() {
    let source = valid_resource_content(&["compound"],"assign_and_while.txt");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_class() {
    let source = valid_resource_content(&["class"],"class.txt");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_collections() {
    let source = valid_resource_content(&["collection"],"collections.txt");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_empty_file() {
    let source = valid_resource_content(&[],"empty_file.txt");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_for_statements() {
    let source = valid_resource_content(&["for_loop"],"for_statements.txt");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_if() {
    let source = valid_resource_content(&["if_expr"],"if.txt");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_tuples() {
    let source = valid_resource_content(&["collection"],"tuples.txt");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_when_statements() {
    let source = valid_resource_content(&["match"],"match_statements.txt");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_while_statements() {
    let source = valid_resource_content(&["while_loop"],"while_statements.txt");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_function_definitions() {
    let source = valid_resource_content(&["function_def"],"function_definitions.txt");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_function_calling() {
    let source = valid_resource_content(&["function_call"],"function_calling.txt");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}

#[test]
fn parse_infix_function_calling() {
    let source = valid_resource_content(&["function_call"],"infix_function_calling.txt");
    assert_ok!(parse(&tokenize(&source).unwrap()));
}
