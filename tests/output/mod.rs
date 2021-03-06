use std::path::Path;
use std::process::Command;

use mamba::pipeline::transpile_directory;

use crate::common::{delete_dir, python_src_to_stmts, resource_content, resource_content_path,
                    resource_content_randomize, resource_path};
use crate::output::common::PYTHON;
use python_parser::ast::Statement;

mod common;

pub mod valid;

fn test_directory(
    valid: bool,
    input: &[&str],
    _output: &[&str],
    file_name: &str
) -> Result<(), Vec<String>> {
    let (output_path, output_file) =
        resource_content_randomize(true, input, &format!("{}.py", file_name));

    let res = fallable(valid, input, &output_path, &output_file, file_name);
    delete_dir(&output_path).map_err(|_| vec![])?;
    let (check_ast, out_ast) = res?;
    assert_eq!(check_ast, out_ast);
    Ok(())
}

fn fallable(
    valid: bool,
    input: &[&str],
    output_path: &str,
    output_file: &str,
    file_name: &str
) -> Result<(Vec<Statement>, Vec<Statement>), Vec<String>> {
    let current_dir_string = resource_path(valid, input, "");
    let current_dir = Path::new(&current_dir_string);

    let map_err = |(ty, msg): &(String, String)| {
        eprintln!("[error | {}] {}", ty, msg);
        format!("[error | {}] {}", ty, msg)
    };
    transpile_directory(&current_dir, Some(&format!("{}.mamba", file_name)), Some(output_path))
        .map_err(|errs| errs.iter().map(&map_err).collect::<Vec<String>>())?;

    let cmd = Command::new(PYTHON)
        .arg("-m")
        .arg("py_compile")
        .arg(&output_file)
        .output()
        .expect("Could not run Python command.");

    if cmd.status.code().unwrap() != 0 {
        panic!("Error running Python command: {}", String::from_utf8(cmd.stderr).unwrap());
    }

    let check_src = resource_content(true, input, &format!("{}_check.py", file_name));
    let check_ast = python_src_to_stmts(&check_src);
    let out_ast = python_src_to_stmts(&resource_content_path(output_file));

    Ok((check_ast, out_ast))
}
