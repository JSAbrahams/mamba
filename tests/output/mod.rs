use std::path::Path;
use std::process::Command;

use mamba::pipeline::transpile_directory;

use crate::common::{delete_dir, python_src_to_stmts, resource_content, resource_content_path,
                    resource_content_randomize, resource_path};
use crate::output::common::PYTHON;

mod common;

pub mod valid;

fn test_directory(
    valid: bool,
    input: &[&str],
    output: &[&str],
    file_name: &str
) -> Result<(), Vec<String>> {
    let current_dir_string = resource_path(valid, input, "");
    let current_dir = Path::new(&current_dir_string);
    let (output_path, output_file) =
        resource_content_randomize(true, input, &format!("{}.py", file_name));

    let map_err = |(ty, msg): &(String, String)| {
        eprintln!("[error | {}] {}", ty, msg);
        format!("[error | {}] {}", ty, msg)
    };
    transpile_directory(
        &current_dir,
        Some(&format!("{}.mamba", file_name)),
        Some(output_path.as_str())
    )
    .map_err(|errs| errs.iter().map(&map_err).collect::<Vec<String>>())?;

    let cmd = Command::new(PYTHON).arg("-m").arg("py_compile").arg(&output_file).output().unwrap();
    if cmd.status.code().unwrap() != 0 {
        panic!("{}", String::from_utf8(cmd.stderr).unwrap());
    }

    let check_src = resource_content(true, input, &format!("{}_check.py", file_name));
    let check_ast = python_src_to_stmts(&check_src);
    let out_ast = python_src_to_stmts(&resource_content_path(&output_file));

    assert_eq!(out_ast, check_ast);
    delete_dir(&output_path).map_err(|_| vec![])
}
