use crate::lexer::tokenize;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use super::*;

fn open_file(file: &str) -> String {
    let mut content = String::new();
    let mut source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    source_path.push("src\\parser\\resources\\".to_owned() + file);

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
fn int() {
    let source = open_file("correct_program_1.txt");
    assert_eq!(true, parse( tokenize(source).unwrap()).is_ok());
}
