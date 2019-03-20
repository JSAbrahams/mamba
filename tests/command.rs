use crate::util::check_valid_resource_exists_and_delete;
use crate::util::valid_resource_path;
use mamba::command::mamba_to_python;
use mamba::command::mamba_to_python_direct;
use std::fs::OpenOptions;
use std::path::Path;

mod util;

#[test]
fn output_class_direct() {
    let source = valid_resource_path(&["class"], "class.mamba");
    let path = &mut Path::new(&source);

    mamba_to_python_direct(path);

    check_valid_resource_exists_and_delete(&["class"], "class.py");
}

#[test]
fn output_class_output_non_existent() {
    let source = valid_resource_path(&["class"], "class.mamba");
    let output = valid_resource_path(&["class"], "class-other.py");

    let path = &mut Path::new(&source);
    let out_path = &mut Path::new(&output);
    mamba_to_python(path, out_path);

    check_valid_resource_exists_and_delete(&["class"], "class-other.py");
}

#[test]
fn output_class_output_exists() {
    let source = valid_resource_path(&["class"], "class.mamba");
    let output = valid_resource_path(&["class"], "class-already-exists.py");

    let path = &mut Path::new(&source);
    let out_path = &mut Path::new(&output);

    OpenOptions::new().write(true).create(true).open(&output); // Create file beforehand
    assert_eq!(true, Path::new(&out_path).exists());

    mamba_to_python(path, out_path);

    check_valid_resource_exists_and_delete(&["class"], "class-already-exists.py");
}

#[test]
fn test_empty_file_direct() {
    let source = valid_resource_path(&[], "empty_file.mamba");
    let path = &mut Path::new(&source);

    mamba_to_python_direct(path);

    check_valid_resource_exists_and_delete(&[], "empty_file.py");
}
