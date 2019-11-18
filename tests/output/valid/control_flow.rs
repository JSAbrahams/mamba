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
fn for_ast_verify() -> Result<(), Vec<(String, String)>> {
    transpile_directory(
        &Path::new(&resource_path(true, &["control_flow"], "")),
        Some("for_statements.mamba"),
        None
    )?;

    let cmd = Command::new(PYTHON)
        .arg("-m")
        .arg("py_compile")
        .arg(resource_path(true, &["control_flow", "target"], "for_statements.py"))
        .output()
        .unwrap();
    if cmd.status.code().unwrap() != 0 {
        panic!("{}", String::from_utf8(cmd.stderr).unwrap());
    }

    let python_src = resource_content(true, &["control_flow"], "for_statements_check.py");
    let out_src = resource_content(true, &["control_flow", "target"], "for_statements.py");

    let python_ast = python_src_to_stmts(&python_src);
    let out_ast = python_src_to_stmts(&out_src);

    assert_eq!(python_ast, out_ast);
    Ok(assert!(exists_and_delete(true, &["control_flow", "target"], "for_statements.py")))
}

#[test]
fn if_ast_verify() -> Result<(), Vec<(String, String)>> {
    transpile_directory(
        &Path::new(&resource_path(true, &["control_flow"], "")),
        Some("if.mamba"),
        None
    )?;

    let cmd = Command::new(PYTHON)
        .arg("-m")
        .arg("py_compile")
        .arg(resource_path(true, &["control_flow", "target"], "if.py"))
        .output()
        .unwrap();
    if cmd.status.code().unwrap() != 0 {
        panic!("{}", String::from_utf8(cmd.stderr).unwrap());
    }

    let python_src = resource_content(true, &["control_flow"], "if_check.py");
    let out_src = resource_content(true, &["control_flow", "target"], "if.py");

    let python_ast = python_src_to_stmts(&python_src);
    let out_ast = python_src_to_stmts(&out_src);

    assert_eq!(python_ast, out_ast);
    Ok(assert!(exists_and_delete(true, &["control_flow", "target"], "if.py")))
}

#[test]
fn while_ast_verify() -> Result<(), Vec<(String, String)>> {
    transpile_directory(
        &Path::new(&resource_path(true, &["control_flow"], "")),
        Some("while.mamba"),
        None
    )?;

    let cmd = Command::new(PYTHON)
        .arg("-m")
        .arg("py_compile")
        .arg(resource_path(true, &["control_flow", "target"], "while.py"))
        .output()
        .unwrap();
    if cmd.status.code().unwrap() != 0 {
        panic!("{}", String::from_utf8(cmd.stderr).unwrap());
    }

    let python_src = resource_content(true, &["control_flow"], "while_check.py");
    let out_src = resource_content(true, &["control_flow", "target"], "while.py");

    let python_ast = python_src_to_stmts(&python_src);
    let out_ast = python_src_to_stmts(&out_src);

    assert_eq!(python_ast, out_ast);
    Ok(assert!(exists_and_delete(true, &["control_flow", "target"], "while.py")))
}

#[test]
fn match_ast_verify() -> Result<(), Vec<(String, String)>> {
    transpile_directory(
        &Path::new(&resource_path(true, &["control_flow"], "")),
        Some("match.mamba"),
        None
    )?;

    let cmd = Command::new(PYTHON)
        .arg("-m")
        .arg("py_compile")
        .arg(resource_path(true, &["control_flow", "target"], "match.py"))
        .output()
        .unwrap();
    if cmd.status.code().unwrap() != 0 {
        panic!("{}", String::from_utf8(cmd.stderr).unwrap());
    }

    let python_src = resource_content(true, &["control_flow"], "match_check.py");
    let out_src = resource_content(true, &["control_flow", "target"], "match.py");

    let python_ast = python_src_to_stmts(&python_src);
    let out_ast = python_src_to_stmts(&out_src);

    assert_eq!(python_ast, out_ast);
    Ok(assert!(exists_and_delete(true, &["control_flow", "target"], "match.py")))
}
