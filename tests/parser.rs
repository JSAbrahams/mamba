use my_lang::lexer::tokenize;
use my_lang::parser::parse;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

fn resource_string_content(file: &str) -> String {
    let mut content = String::new();
    let mut source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    source_path.push("tests\\resources\\".to_owned() + file);

    match source_path.to_str() {
        Some(path) => match File::open(path) {
            Ok(mut file) => { file.read_to_string(&mut content).unwrap(); }
            Err(error) => { panic!("Error opening file {}: {}", path, error) }
        }
        None => panic!("Error opening file: path can't be converted to string.")
    }

    return content;
}

#[test]
fn parse_assigns_and_while() {
    let source = resource_string_content("assign_and_while.txt");
    parse(tokenize(source).unwrap()).unwrap();
}

#[test]
fn parse_if_statements() {
    let source = resource_string_content("if_statements.txt");
    parse(tokenize(source).unwrap()).unwrap();
}

#[test]
fn parse_tuples() {
    let source = resource_string_content("tuples.txt");
    println!("{:?}", parse(tokenize(source).unwrap()).unwrap());
}
