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
fn correct_1() {
    let source = resource_string_content("correct_program_1.txt");
    parse(tokenize(source).unwrap()).unwrap();
}

#[test]
fn correct_2() {
    let source = resource_string_content("correct_program_2.txt");
    parse(tokenize(source).unwrap()).unwrap();
}
