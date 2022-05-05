use std::fmt::{Debug, Formatter};
use std::path::Path;
use std::process::Command;

use python_parser::ast::Statement;

use mamba::common::delimit::newline_delimited;
use mamba::pipeline::transpile_directory;

use crate::common::{delete_dir, python_src_to_stmts, resource_content, resource_content_path,
                    resource_content_randomize, resource_path};
use crate::system::common::PYTHON;

mod common;

pub mod valid;

struct OutTestErr(Vec<String>);

type OutTestRet<T = ()> = Result<T, OutTestErr>;

impl Debug for OutTestErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.iter().map(|err| write!(f, "{}\n", err)).collect()
    }
}

fn test_directory(
    valid: bool,
    input: &[&str],
    _output: &[&str],
    file_name: &str,
) -> OutTestRet {
    let (output_path, output_file) =
        resource_content_randomize(true, input, &format!("{}.py", file_name));

    let res = fallable(valid, input, &output_path, &output_file, file_name);
    delete_dir(&output_path).map_err(|_| OutTestErr(vec![]))?;
    let (check_ast, check_src, out_ast, out_src) = res?;

    // Convert to newline delimited string for more readable diff
    let check_string = newline_delimited(check_ast.iter().map(|stmt| format!("{:?}", stmt)));
    let out_string = newline_delimited(out_ast.iter().map(|stmt| format!("{:?}", stmt)));
    assert_eq!(out_string, check_string,
               "AST did not match!\n\
               Was:\n\
               ----------------\n\
               {}\n\
               ----------------\n\
               Expected:\n\
               ----------------\n\
               {}\n\
               ----------------", out_src, check_src);
    Ok(())
}

fn fallable(
    valid: bool,
    input: &[&str],
    output_path: &str,
    output_file: &str,
    file_name: &str,
) -> OutTestRet<(Vec<Statement>, String, Vec<Statement>, String)> {
    let current_dir_string = resource_path(valid, input, "");
    let current_dir = Path::new(&current_dir_string);

    let map_err = |(ty, msg): &(String, String)| {
        format!("[error | {}] {}", ty, msg)
    };
    transpile_directory(&current_dir, Some(&format!("{}.mamba", file_name)), Some(output_path))
        .map_err(|errs| OutTestErr(errs.iter().map(&map_err).collect::<Vec<String>>()))?;

    let cmd = Command::new(PYTHON)
        .arg("-m")
        .arg("py_compile")
        .arg(&output_file)
        .output()
        .expect("Could not run Python command.");


    let check_src = resource_content(true, input, &format!("{}_check.py", file_name));
    let check_ast = python_src_to_stmts(&check_src);
    let out_src = resource_content_path(output_file);
    let out_ast = python_src_to_stmts(&out_src);

    if cmd.status.code().unwrap() != 0 {
        panic!("Error running Python command: {}\n\
        Source:\n\
        ----------\n\
        {}\n\
        ----------",
               String::from_utf8(cmd.stderr).unwrap(),
               out_src.lines().enumerate().map(|(line, src)| {
                   format!("{}: {}\n", line + 1, src)
               }).collect::<String>());
    }

    Ok((check_ast, check_src, out_ast, out_src))
}
