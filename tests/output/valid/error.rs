extern crate python_parser;

use crate::common::exists_and_delete;
use crate::common::python_src_to_stmts;
use crate::common::resource_content;
use crate::common::resource_path;
use crate::output::common::PYTHON;
use mamba::pipeline::transpile_directory;
use std::path::Path;
use std::process::Command;

#[test]
fn handle_ast_verify() -> Result<(), Vec<(String, String)>> {
    transpile_directory(
        &Path::new(&resource_path(true, &["error"], "")),
        Some("handle.mamba"),
        None
    )?;

    let cmd = Command::new(PYTHON)
        .arg("-m")
        .arg("py_compile")
        .arg(resource_path(true, &["error", "target"], "handle.py"))
        .output()
        .unwrap();
    if cmd.status.code().unwrap() != 0 {
        panic!("{}", String::from_utf8(cmd.stderr).unwrap());
    }

    let python_src = resource_content(true, &["error"], "handle_check.py");
    let out_src = resource_content(true, &["error", "target"], "handle.py");

    let python_ast = python_src_to_stmts(&python_src);
    let out_ast = python_src_to_stmts(&out_src);

    assert_eq!(out_ast, python_ast);
    Ok(assert!(exists_and_delete(true, &["error", "target"], "handle.py")))
}

#[test]
#[ignore]
fn raise_ast_verify() -> Result<(), Vec<(String, String)>> {
    transpile_directory(
        &Path::new(&resource_path(true, &["error"], "")),
        Some("raise.mamba"),
        None
    )?;

    let cmd = Command::new(PYTHON)
        .arg("-m")
        .arg("py_compile")
        .arg(resource_path(true, &["error", "target"], "raise.py"))
        .output()
        .unwrap();
    if cmd.status.code().unwrap() != 0 {
        panic!("{}", String::from_utf8(cmd.stderr).unwrap());
    }

    let python_src = resource_content(true, &["error"], "raise_check.py");
    let out_src = resource_content(true, &["error", "target"], "raise.py");

    let python_ast = python_src_to_stmts(&python_src);
    let out_ast = python_src_to_stmts(&out_src);

    assert_eq!(out_ast, python_ast);
    Ok(assert!(exists_and_delete(true, &["error", "target"], "raise.py")))
}

#[test]
fn with_ast_verify() -> Result<(), Vec<(String, String)>> {
    transpile_directory(
        &Path::new(&resource_path(true, &["error"], "")),
        Some("with.mamba"),
        None
    )?;

    let cmd = Command::new(PYTHON)
        .arg("-m")
        .arg("py_compile")
        .arg(resource_path(true, &["error", "target"], "with.py"))
        .output()
        .unwrap();
    if cmd.status.code().unwrap() != 0 {
        panic!("{}", String::from_utf8(cmd.stderr).unwrap());
    }

    let python_src = resource_content(true, &["error"], "with_check.py");
    let out_src = resource_content(true, &["error", "target"], "with.py");

    let python_ast = python_src_to_stmts(&python_src);
    let out_ast = python_src_to_stmts(&out_src);

    assert_eq!(out_ast, python_ast);
    Ok(assert!(exists_and_delete(true, &["error", "target"], "with.py")))
}
