use crate::util::*;
use mamba::command::mamba_to_python_direct;
use std::path::Path;
use std::process::Command;

#[test]
fn output_tuple_valid_syntax() {
    let source = valid_resource_path(&["collection"], "tuple.mamba");
    let path = mamba_to_python_direct(Path::new(&source)).unwrap();
    let python = if cfg!(windows) { "py" } else { "python3" };
    let cmd = Command::new(python).arg("-m").arg("py_compile").arg(path).output().unwrap();

    if cmd.status.code().unwrap() != 0 {
        panic!("{}", String::from_utf8(cmd.stderr).unwrap());
    }
}

#[test]
#[ignore]
fn output_class_valid_syntax() {
    let source = valid_resource_path(&["class"], "class.mamba");
    let path = mamba_to_python_direct(Path::new(&source)).unwrap();
    let python = if cfg!(windows) { "py" } else { "python3" };
    let cmd = Command::new(python).arg("-m").arg("py_compile").arg(path).output().unwrap();

    if cmd.status.code().unwrap() != 0 {
        panic!("{}", String::from_utf8(cmd.stderr).unwrap());
    }
}
