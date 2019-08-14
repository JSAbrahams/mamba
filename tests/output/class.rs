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
fn generics_ast_verify() -> Result<(), Vec<(String, String)>> {
    transpile_directory(
        &Path::new(&resource_path(true, &["class"], "")),
        Some("generics.mamba"),
        None
    )?;
    let cmd = Command::new(PYTHON)
        .arg("-m")
        .arg("py_compile")
        .arg(resource_path(true, &["class", "target"], "generics.py"))
        .output()
        .unwrap();
    if cmd.status.code().unwrap() != 0 {
        panic!("{}", String::from_utf8(cmd.stderr).unwrap());
    }

    let python_src = resource_content(true, &["class"], "generics_check.py");
    let out_src = resource_content(true, &["class", "target"], "generics.py");

    let python_ast = python_src_to_stmts(&python_src);
    let out_ast = python_src_to_stmts(&out_src);

    assert_eq!(python_ast, out_ast);
    Ok(assert!(exists_and_delete(true, &["class", "target"], "generics.py")))
}

#[test]
fn import_ast_verify() -> Result<(), Vec<(String, String)>> {
    transpile_directory(
        &Path::new(resource_path(true, &["class"], "").as_str()),
        Some("import.mamba"),
        None
    )?;

    let cmd = Command::new(PYTHON)
        .arg("-m")
        .arg("py_compile")
        .arg(resource_path(true, &["class", "target"], "import.py"))
        .output()
        .unwrap();
    if cmd.status.code().unwrap() != 0 {
        panic!("{}", String::from_utf8(cmd.stderr).unwrap());
    }

    let python_src = resource_content(true, &["class"], "import_check.py");
    let out_src = resource_content(true, &["class", "target"], "import.py");

    let python_ast = python_src_to_stmts(&python_src);
    let out_ast = python_src_to_stmts(&out_src);

    assert_eq!(out_ast, python_ast);
    Ok(assert!(exists_and_delete(true, &["class", "target"], "import.py")))
}

#[test]
fn parent_ast_verify() -> Result<(), Vec<(String, String)>> {
    transpile_directory(
        &Path::new(resource_path(true, &["class"], "").as_str()),
        Some("parent.mamba"),
        None
    )?;

    let cmd = Command::new(PYTHON)
        .arg("-m")
        .arg("py_compile")
        .arg(resource_path(true, &["class", "target"], "parent.py"))
        .output()
        .unwrap();
    if cmd.status.code().unwrap() != 0 {
        panic!("{}", String::from_utf8(cmd.stderr).unwrap());
    }

    let python_src = resource_content(true, &["class"], "parent_check.py");
    let out_src = resource_content(true, &["class", "target"], "parent.py");

    let python_ast = python_src_to_stmts(&python_src);
    let out_ast = python_src_to_stmts(&out_src);

    assert_eq!(python_ast, out_ast);
    Ok(assert!(exists_and_delete(true, &["class", "target"], "parent.py")))
}

#[test]
fn types_ast_verify() -> Result<(), Vec<(String, String)>> {
    transpile_directory(
        &Path::new(resource_path(true, &["class"], "").as_str()),
        Some("types.mamba"),
        None
    )?;

    let cmd = Command::new(PYTHON)
        .arg("-m")
        .arg("py_compile")
        .arg(resource_path(true, &["class", "target"], "types.py"))
        .output()
        .unwrap();
    if cmd.status.code().unwrap() != 0 {
        panic!("{}", String::from_utf8(cmd.stderr).unwrap());
    }

    let python_src = resource_content(true, &["class"], "types_check.py");
    let out_src = resource_content(true, &["class", "target"], "types.py");

    let python_ast = python_src_to_stmts(&python_src);
    let out_ast = python_src_to_stmts(&out_src);

    assert_eq!(python_ast, out_ast);
    Ok(assert!(exists_and_delete(true, &["class", "target"], "types.py")))
}
