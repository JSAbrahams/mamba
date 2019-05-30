extern crate python_parser;
use crate::common::exists_and_delete;
use crate::common::python_src_to_stmts;
use crate::common::resource_content;
use crate::common::resource_path;
use crate::output::common::PYTHON;
use mamba::pipeline::mamba_to_python;
use std::path::Path;
use std::process::Command;

#[test]
fn handle_ast_verify() {
    let mamba_path = resource_path(true, &["error"], "handle.mamba");
    let out_path = mamba_to_python(Path::new(&mamba_path), None).unwrap();

    let cmd = Command::new(PYTHON).arg("-m").arg("py_compile").arg(out_path).output().unwrap();
    if cmd.status.code().unwrap() != 0 {
        panic!("{}", String::from_utf8(cmd.stderr).unwrap());
    }

    let python_src = resource_content(true, &["error"], "handle_check.py");
    let out_src = resource_content(true, &["error"], "handle.py");

    let python_ast = python_src_to_stmts(&python_src);
    let out_ast = python_src_to_stmts(&out_src);

    assert_eq!(python_ast, out_ast);
    exists_and_delete(true, &["error"], "handle.py");
}

#[test]
fn raise_ast_verify() {
    let mamba_path = resource_path(true, &["error"], "raise.mamba");
    let out_path = mamba_to_python(Path::new(&mamba_path), None).unwrap();

    let cmd = Command::new(PYTHON).arg("-m").arg("py_compile").arg(out_path).output().unwrap();
    if cmd.status.code().unwrap() != 0 {
        panic!("{}", String::from_utf8(cmd.stderr).unwrap());
    }

    let python_src = resource_content(true, &["error"], "raise_check.py");
    let out_src = resource_content(true, &["error"], "raise.py");

    let python_ast = python_src_to_stmts(&python_src);
    let out_ast = python_src_to_stmts(&out_src);

    assert_eq!(python_ast, out_ast);
    exists_and_delete(true, &["error"], "raise.py");
}
