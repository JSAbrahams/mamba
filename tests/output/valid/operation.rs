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
fn arithmetic_ast_verify() -> Result<(), Vec<(String, String)>> {
    transpile_directory(
        &Path::new(&resource_path(true, &["operation"], "")),
        Some("arithmetic.mamba"),
        None
    )?;

    let cmd = Command::new(PYTHON)
        .arg("-m")
        .arg("py_compile")
        .arg(resource_path(true, &["operation", "target"], "arithmetic.py"))
        .output()
        .unwrap();
    if cmd.status.code().unwrap() != 0 {
        panic!("{}", String::from_utf8(cmd.stderr).unwrap());
    }

    let python_src = resource_content(true, &["operation"], "arithmetic_check.py");
    let out_src = resource_content(true, &["operation", "target"], "arithmetic.py");

    let python_ast = python_src_to_stmts(&python_src);
    let out_ast = python_src_to_stmts(&out_src);

    assert_eq!(python_ast, out_ast);
    Ok(assert!(exists_and_delete(true, &["operation", "target"], "arithmetic.py")))
}

#[test]
fn primitives_ast_verify() -> Result<(), Vec<(String, String)>> {
    transpile_directory(
        &Path::new(&resource_path(true, &["operation"], "")),
        Some("primitives.mamba"),
        None
    )?;

    let cmd = Command::new(PYTHON)
        .arg("-m")
        .arg("py_compile")
        .arg(resource_path(true, &["operation", "target"], "primitives.py"))
        .output()
        .unwrap();
    if cmd.status.code().unwrap() != 0 {
        panic!("{}", String::from_utf8(cmd.stderr).unwrap());
    }

    let python_src = resource_content(true, &["operation"], "primitives_check.py");
    let out_src = resource_content(true, &["operation", "target"], "primitives.py");

    let python_ast = python_src_to_stmts(&python_src);
    let out_ast = python_src_to_stmts(&out_src);

    assert_eq!(python_ast, out_ast);
    Ok(assert!(exists_and_delete(true, &["operation", "target"], "primitives.py")))
}

#[test]
fn bitwise_ast_verify() -> Result<(), Vec<(String, String)>> {
    transpile_directory(
        &Path::new(&resource_path(true, &["operation"], "")),
        Some("bitwise.mamba"),
        None
    )?;

    let cmd = Command::new(PYTHON)
        .arg("-m")
        .arg("py_compile")
        .arg(resource_path(true, &["operation", "target"], "bitwise.py"))
        .output()
        .unwrap();
    if cmd.status.code().unwrap() != 0 {
        panic!("{}", String::from_utf8(cmd.stderr).unwrap());
    }

    let python_src = resource_content(true, &["operation"], "bitwise_check.py");
    let out_src = resource_content(true, &["operation", "target"], "bitwise.py");

    let python_ast = python_src_to_stmts(&python_src);
    let out_ast = python_src_to_stmts(&out_src);

    assert_eq!(python_ast, out_ast);
    Ok(assert!(exists_and_delete(true, &["operation", "target"], "bitwise.py")))
}

#[test]
fn boolean_ast_verify() -> Result<(), Vec<(String, String)>> {
    transpile_directory(
        &Path::new(&resource_path(true, &["operation"], "")),
        Some("boolean.mamba"),
        None
    )?;

    let cmd = Command::new(PYTHON)
        .arg("-m")
        .arg("py_compile")
        .arg(resource_path(true, &["operation", "target"], "boolean.py"))
        .output()
        .unwrap();
    if cmd.status.code().unwrap() != 0 {
        panic!("{:#?}", String::from_utf8(cmd.stderr).unwrap());
    }

    let python_src = resource_content(true, &["operation"], "boolean_check.py");
    let out_src = resource_content(true, &["operation", "target"], "boolean.py");

    let python_ast = python_src_to_stmts(&python_src);
    let out_ast = python_src_to_stmts(&out_src);

    assert_eq!(out_ast, python_ast);
    Ok(assert!(exists_and_delete(true, &["operation", "target"], "boolean.py")))
}
