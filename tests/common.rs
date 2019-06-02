extern crate python_parser;

use python_parser::ast::Statement;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

#[allow(dead_code)]
pub fn resource_content(valid: bool, subdirs: &[&str], file: &str) -> String {
    match File::open(resource_path(valid, subdirs, file)) {
        Ok(mut path) => {
            let mut content = String::new();
            match path.read_to_string(&mut content) {
                Ok(_) => content,
                Err(err) => panic!("Error while reading file contents: {}.", err)
            }
        }
        Err(err) =>
            panic!("Error while opening file {} while reading resource contents: {}.", file, err),
    }
}

#[allow(dead_code)]
pub fn resource_path(valid: bool, subdirs: &[&str], file: &str) -> String {
    let mut source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("resources")
        .join(if valid { "valid" } else { "invalid" });

    for dir in subdirs {
        source_path = source_path.join(dir);
    }

    source_path = source_path.join(file);
    String::from(source_path.to_string_lossy())
}

#[allow(dead_code)]
pub fn exists_and_delete(valid: bool, subdirs: &[&str], file: &str) -> bool {
    let resource_path = resource_path(valid, subdirs, file);
    let path = Path::new(&resource_path);
    if !path.exists() {
        return false;
    }

    match fs::remove_file(path) {
        Ok(_) => true,
        Err(err) => panic!("{}: {}", err, path.display())
    }
}

#[allow(dead_code)]
pub fn python_src_to_stmts(python_src: &String) -> Vec<Statement> {
    python_parser::file_input(python_parser::make_strspan(python_src.as_ref())).unwrap().1
}
