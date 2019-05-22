use crate::common::check_exists_and_delete;
use crate::common::resource_path;
use mamba::command::mamba_to_python;
use mamba::command::mamba_to_python_direct;
use std::fs::OpenOptions;
use std::path::Path;

mod common;

#[test]
fn output_class_direct_valid_syntax() {
    let source = resource_path(true, &["class"], "class.mamba");
    let path = &mut Path::new(&source);

    match mamba_to_python_direct(path) {
        Ok(_) => check_exists_and_delete(true, &["class"], "class.py"),
        Err(err) => panic!("{}", err)
    };
}

#[test]
fn output_class_output_non_existent() {
    let source = resource_path(true, &["class"], "class.mamba");
    let output = resource_path(true, &["class"], "class-other.py");

    let path = &mut Path::new(&source);
    let out_path = &mut Path::new(&output);
    match mamba_to_python(path, out_path) {
        Ok(_) => check_exists_and_delete(true, &["class"], "class-other.py"),
        Err(err) => panic!("{}", err)
    };
}

#[test]
fn output_class_output_exists() {
    let source = resource_path(true, &["class"], "class.mamba");
    let output = resource_path(true, &["class"], "class-already-exists.py");

    let path = &mut Path::new(&source);
    let out_path = &mut Path::new(&output);

    match OpenOptions::new().write(true).create(true).open(&output) {
        Ok(_) => assert_eq!(true, Path::new(&out_path).exists()),
        Err(err) => panic!("{}", err)
    };

    match mamba_to_python(path, out_path) {
        Ok(_) => check_exists_and_delete(true, &["class"], "class-already-exists.py"),
        Err(err) => panic!("{}", err)
    };
}

#[test]
fn test_empty_file_direct() {
    let source = resource_path(true, &[], "empty_file.mamba");
    let path = &mut Path::new(&source);

    match mamba_to_python_direct(path) {
        Ok(_) => check_exists_and_delete(true, &[], "empty_file_check.py"),
        Err(err) => panic!("{}", err)
    };
}
