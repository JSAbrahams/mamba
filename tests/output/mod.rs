use crate::common::{exists_and_delete, python_src_to_stmts, resource_content, resource_path};
use crate::output::common::PYTHON;
use mamba::pipeline::transpile_directory;
use std::path::Path;
use std::process::Command;

mod common;

pub mod valid;

fn test_directory(
    valid: bool,
    input: &[&str],
    output: &[&str],
    file_name: &str
) -> Result<(), Vec<String>> {
    transpile_directory(
        &Path::new(&resource_path(valid, input, "")),
        Some(&format!("{}.mamba", file_name)),
        None
    )
    .map_err(|errs| {
        errs.iter()
            .map(|(ty, msg)| {
                eprintln!("[{}] {}", ty, msg);
                format!("[{}] {}", ty, msg)
            })
            .collect::<Vec<String>>()
    })?;

    let cmd = Command::new(PYTHON)
        .arg("-m")
        .arg("py_compile")
        .arg(resource_path(true, output, &format!("{}.py", file_name)))
        .output()
        .unwrap();
    if cmd.status.code().unwrap() != 0 {
        panic!("{}", String::from_utf8(cmd.stderr).unwrap());
    }

    let python_src = resource_content(true, input, &format!("{}_check.py", file_name));
    let out_src = resource_content(true, output, &format!("{}.py", file_name));
    let python_ast = python_src_to_stmts(&python_src);
    let out_ast = python_src_to_stmts(&out_src);

    assert_eq!(out_ast, python_ast);
    Ok(assert!(exists_and_delete(true, output, &format!("{}.py", file_name))))
}
