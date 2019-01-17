use my_lang::lexer::tokenize;
use my_lang::parser::parse;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[macro_use]
macro_rules! assert_ok { ( $ expr: expr) => {{
    match $ expr {
        Ok(r) => println!("{:?}",r),
        Err(err) => panic ! ("{}", err)
    }
}}}

fn resource_string_content(file: String) -> String {
    let mut content = String::new();
    let mut source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    source_path.push("tests\\resources\\".to_owned());
    source_path.push(file);

    match source_path.to_str() {
        Some(path) => match File::open(path) {
            Ok(mut file) => { file.read_to_string(&mut content).unwrap(); }
            Err(error) => { panic!("Error opening file {}: {}", path, error) }
        }
        None => panic!("Error opening file: path can't be converted to string.")
    }

    return content;
}

fn valid_resource(file: &str) -> String { resource_string_content("valid\\".to_owned() + file) }

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
    println!("{:?}", tokenize(source));
//    assert_ok!(parse(tokenize(source).unwrap()));
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
