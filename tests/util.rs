use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[macro_export]
macro_rules! assert_ok { ( $ expr: expr) => {{
    match $ expr {
        Ok(r) => (),
        Err(err) => panic ! ("{}", err)
    }
}}}

pub fn resource_string_content(file: String) -> String {
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

pub fn valid_resource(file: &str) -> String { resource_string_content("valid\\".to_owned() + file) }
