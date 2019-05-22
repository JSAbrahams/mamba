use crate::common::*;
use crate::output::common::PYTHON;
use mamba::command::mamba_to_python_direct;
use std::path::Path;
use std::process::Command;

#[test]
fn output_tuple_valid_syntax() {
    let source = resource_path(true, &["collection"], "tuple.mamba");
    let path = mamba_to_python_direct(Path::new(&source)).unwrap();

    let cmd = Command::new(PYTHON).arg("-m").arg("py_compile").arg(path).output().unwrap();
    if cmd.status.code().unwrap() != 0 {
        panic!("{}", String::from_utf8(cmd.stderr).unwrap());
    }
    check_exists_and_delete(true, &["collection"], "tuple.py");
}

#[test]
#[ignore]
fn output_class_valid_syntax() {
    let source = resource_path(true, &["class"], "class.mamba");
    let path = mamba_to_python_direct(Path::new(&source)).unwrap();

    let cmd = Command::new(PYTHON).arg("-m").arg("py_compile").arg(path).output().unwrap();
    if cmd.status.code().unwrap() != 0 {
        panic!("{}", String::from_utf8(cmd.stderr).unwrap());
    }
    check_exists_and_delete(true, &["class"], "class.py");
}
