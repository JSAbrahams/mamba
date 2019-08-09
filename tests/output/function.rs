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
fn call_ast_verify() -> Result<(), Vec<(String, String)>> {
    transpile_directory(
        &Path::new(&resource_path(true, &["function"], "")),
        Some("calls.mamba"),
        None
    )?;

    let cmd = Command::new(PYTHON)
        .arg("-m")
        .arg("py_compile")
        .arg(resource_path(true, &["function", "target"], "calls.py"))
        .output()
        .unwrap();
    if cmd.status.code().unwrap() != 0 {
        panic!("{}", String::from_utf8(cmd.stderr).unwrap());
    }

    let python_src = resource_content(true, &["function"], "calls_check.py");
    let out_src = resource_content(true, &["function", "target"], "calls.py");

    let python_ast = python_src_to_stmts(&python_src);
    let out_ast = python_src_to_stmts(&out_src);

    assert_eq!(python_ast, out_ast);
    Ok(assert!(exists_and_delete(true, &["function", "target"], "calls.py")))
}

#[test]
fn definition_ast_verify() -> Result<(), Vec<(String, String)>> {
    transpile_directory(
        &Path::new(&resource_path(true, &["function"], "")),
        Some("definition.mamba"),
        None
    )?;

    let cmd = Command::new(PYTHON)
        .arg("-m")
        .arg("py_compile")
        .arg(resource_path(true, &["function", "target"], "definition.py"))
        .output()
        .unwrap();
    if cmd.status.code().unwrap() != 0 {
        panic!("{}", String::from_utf8(cmd.stderr).unwrap());
    }

    let python_src = resource_content(true, &["function"], "definition_check.py");
    let out_src = resource_content(true, &["function", "target"], "definition.py");

    let python_ast = python_src_to_stmts(&python_src);
    let out_ast = python_src_to_stmts(&out_src);

    assert_eq!(python_ast, out_ast);
    Ok(assert!(exists_and_delete(true, &["function", "target"], "definition.py")))
}

#[test]
fn infix_calls_ast_verify() -> Result<(), Vec<(String, String)>> {
    transpile_directory(
        &Path::new(resource_path(true, &["function"], "").as_str()),
        Some("infix_calls.mamba"),
        None
    )?;

    let cmd = Command::new(PYTHON)
        .arg("-m")
        .arg("py_compile")
        .arg(resource_path(true, &["function", "target"], "infix_calls.py"))
        .output()
        .unwrap();
    if cmd.status.code().unwrap() != 0 {
        panic!("{}", String::from_utf8(cmd.stderr).unwrap());
    }

    let python_src = resource_content(true, &["function"], "infix_calls_check.py");
    let out_src = resource_content(true, &["function", "target"], "infix_calls.py");

    let python_ast = python_src_to_stmts(&python_src);
    let out_ast = python_src_to_stmts(&out_src);

    assert_eq!(python_ast, out_ast);
    Ok(assert!(exists_and_delete(true, &["function", "target"], "infix_calls.py")))
}
