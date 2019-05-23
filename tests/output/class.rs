extern crate python_parser;

use crate::common::check_exists_and_delete;
use crate::common::python_src_to_stmts;
use crate::common::resource_content;
use crate::common::resource_path;
use crate::output::common::PYTHON;
use mamba::command::mamba_to_python_direct;
use std::path::Path;
use std::process::Command;

#[test]
fn generics_ast_verify() {
    let mamba_path = resource_path(true, &["class"], "generics.mamba");
    let out_path = mamba_to_python_direct(Path::new(&mamba_path)).unwrap();

    let cmd = Command::new(PYTHON).arg("-m").arg("py_compile").arg(out_path).output().unwrap();
    if cmd.status.code().unwrap() != 0 {
        panic!("{}", String::from_utf8(cmd.stderr).unwrap());
    }

    let python_src = resource_content(true, &["class"], "generics_check.py");
    let out_src = resource_content(true, &["class"], "generics.py");

    let python_ast = python_src_to_stmts(&python_src);
    let out_ast = python_src_to_stmts(&out_src);

    assert_eq!(python_ast, out_ast);
    check_exists_and_delete(true, &["class"], "generics.py");
}

#[test]
fn import_ast_verify() {
    let mamba_path = resource_path(true, &["class"], "import.mamba");
    let out_path = mamba_to_python_direct(Path::new(&mamba_path)).unwrap();

    let cmd = Command::new(PYTHON).arg("-m").arg("py_compile").arg(out_path).output().unwrap();
    if cmd.status.code().unwrap() != 0 {
        panic!("{}", String::from_utf8(cmd.stderr).unwrap());
    }

    let python_src = resource_content(true, &["class"], "import_check.py");
    let out_src = resource_content(true, &["class"], "import.py");

    let python_ast = python_src_to_stmts(&python_src);
    let out_ast = python_src_to_stmts(&out_src);

    assert_eq!(python_ast, out_ast);
    check_exists_and_delete(true, &["class"], "import.py");
}

#[test]
fn parent_ast_verify() {
    let mamba_path = resource_path(true, &["class"], "parent.mamba");
    let out_path = mamba_to_python_direct(Path::new(&mamba_path)).unwrap();

    let cmd = Command::new(PYTHON).arg("-m").arg("py_compile").arg(out_path).output().unwrap();
    if cmd.status.code().unwrap() != 0 {
        panic!("{}", String::from_utf8(cmd.stderr).unwrap());
    }

    let python_src = resource_content(true, &["class"], "parent_check.py");
    let out_src = resource_content(true, &["class"], "parent.py");

    let python_ast = python_src_to_stmts(&python_src);
    let out_ast = python_src_to_stmts(&out_src);

    assert_eq!(python_ast, out_ast);
    check_exists_and_delete(true, &["class"], "parent.py");
}

#[test]
fn types_ast_verify() {
    let mamba_path = resource_path(true, &["class"], "types.mamba");
    let out_path = mamba_to_python_direct(Path::new(&mamba_path)).unwrap();

    let cmd = Command::new(PYTHON).arg("-m").arg("py_compile").arg(out_path).output().unwrap();
    if cmd.status.code().unwrap() != 0 {
        panic!("{}", String::from_utf8(cmd.stderr).unwrap());
    }

    let python_src = resource_content(true, &["class"], "types_check.py");
    let out_src = resource_content(true, &["class"], "types.py");

    let python_ast = python_src_to_stmts(&python_src);
    let out_ast = python_src_to_stmts(&out_src);

    assert_eq!(python_ast, out_ast);
    check_exists_and_delete(true, &["class"], "types.py");
}
