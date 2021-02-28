extern crate python_parser;

use std::fs;
use std::fs::{create_dir, File};
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use python_parser::ast::Statement;
use tempfile::tempdir_in;

/// Get contents of file of given path as string.
pub fn resource_content_path(path: &str) -> String {
    match File::open(path) {
        Ok(mut path) => {
            let mut content = String::new();
            match path.read_to_string(&mut content) {
                Ok(_) => content,
                Err(err) => panic!("Error while reading file contents: {}.", err)
            }
        }
        Err(err) =>
            panic!("Error while opening file {} while reading resource contents: {}.", path, err),
    }
}

/// Get the path of a file at a given location.
///
/// * `valid` - Whether this is a happy or a sad path. See how test resources are structured.
/// * `subdirs` - Path to directory of resource under test.
/// * `file` - Name of file under test.
///
/// Returns:
/// - The absolute path of the resource, or the directory, as a string.
/// - The absolute path of the random output directory or file, to be deleted after the test.
pub fn resource_content_randomize(valid: bool, subdirs: &[&str], file: &str) -> (String, String) {
    let mut source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("resource")
        .join(if valid { "valid" } else { "invalid" });
    for dir in subdirs {
        source_path = source_path.join(dir);
    }

    if !source_path.exists() {
        create_dir(&source_path).expect(format!("Path not found: {}",
                                                source_path.as_path().display()).as_str());
    }

    let source_path = tempdir_in(source_path.clone())
        .expect(format!("Could not create temp dir: {}",
                        source_path.display()).as_str());
    let source = source_path.path();

    if file.is_empty() {
        (String::from(source.to_string_lossy()), String::from(source.to_string_lossy()))
    } else {
        (String::from(source.to_string_lossy()), String::from(source.join(file).to_string_lossy()))
    }
}

pub fn resource_content(valid: bool, subdirs: &[&str], file: &str) -> String {
    resource_content_path(&resource_path(valid, subdirs, file))
}

pub fn resource_path(valid: bool, subdirs: &[&str], file: &str) -> String {
    let mut source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("resource")
        .join(if valid { "valid" } else { "invalid" });
    for dir in subdirs {
        source_path = source_path.join(dir);
    }

    if !source_path.exists() {
        create_dir(&source_path).expect(format!("Path not found: {}",
                                                source_path.as_path().display()).as_str());
    }

    source_path = source_path.join(file);
    String::from(source_path.to_string_lossy())
}

pub fn delete_dir(resource_path: &String) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(&resource_path);
    if !path.exists() {
        panic!("{} does not exist", path.display())
    } else {
        match fs::remove_dir_all(path) {
            Ok(_) => Ok(()),
            Err(err) => panic!("[{}] {}", err, path.display())
        }
    }
}

pub fn python_src_to_stmts(python_src: &String) -> Vec<Statement> {
    python_parser::file_input(python_parser::make_strspan(python_src.as_ref())).unwrap().1
}
