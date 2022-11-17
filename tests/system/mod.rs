use std::cmp::max;
use std::fmt::{Debug, Formatter};
use std::path::Path;
use std::process::Command;

use itertools::{EitherOrBoth, Itertools};
use python_parser::ast::Statement;

use mamba::{Arguments, transpile_dir};
use mamba::common::delimit::newline_delimited;

use crate::common::{
    delete_dir, python_src_to_stmts, resource_content, resource_content_path,
    resource_content_randomize, resource_path,
};
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

/// Test directory with default set to annotate output.
fn test_directory(valid: bool, input: &[&str], output: &[&str], file_name: &str) -> OutTestRet {
    let args = Arguments { annotate: true };
    test_directory_args(valid, input, output, file_name, &args)
}

fn test_directory_args(valid: bool, input: &[&str], _: &[&str], file_name: &str, args: &Arguments) -> OutTestRet {
    let (output_path, output_file) =
        resource_content_randomize(true, input, &format!("{}.py", file_name));

    let res = fallable(valid, input, &output_path, &output_file, file_name, args);
    delete_dir(&output_path).map_err(|_| OutTestErr(vec![]))?;
    let (check_ast, check_src, out_ast, out_src) = res?;

    // Convert to newline delimited string for more readable diff
    let check_string = newline_delimited(check_ast.iter().map(|stmt| format!("{:?}", stmt)));
    let out_string = newline_delimited(out_ast.iter().map(|stmt| format!("{:?}", stmt)));

    let longest_line = out_src.lines().max_by(|l1, l2| l1.len().cmp(&l2.len())).unwrap_or("").len();
    let min_line = 25;
    let out_line_len = max(min_line, longest_line);

    let gap = 12;
    let sep_count = 10;
    let mut msg = format!(
        "Was AST:{}Expected AST:\n{}{}{}\n",
        String::from_utf8(vec![b' '; out_line_len - 8 + gap]).unwrap(),
        String::from_utf8(vec![b'-'; sep_count]).unwrap(),
        String::from_utf8(vec![b' '; out_line_len - sep_count + gap]).unwrap(),
        String::from_utf8(vec![b'-'; sep_count]).unwrap()
    );
    for line in out_src.lines().zip_longest(check_src.lines()) {
        match line {
            EitherOrBoth::Both(out, check) => {
                let left_len = out.len();
                msg.push_str(&format!(
                    "{}{}{}\n",
                    out,
                    String::from_utf8(vec![b' '; out_line_len + gap - left_len]).unwrap(),
                    check
                ))
            }
            EitherOrBoth::Left(out) => msg.push_str(&format!("{}\n", out)),
            EitherOrBoth::Right(check) => msg.push_str(&format!(
                "{}{}\n",
                String::from_utf8(vec![b' '; out_line_len + gap]).unwrap(),
                check
            )),
        }
    }
    assert_eq!(out_string, check_string, "{}", msg);
    Ok(())
}

fn fallable(
    valid: bool,
    input: &[&str],
    output_path: &str,
    output_file: &str,
    file_name: &str,
    arguments: &Arguments,
) -> OutTestRet<(Vec<Statement>, String, Vec<Statement>, String)> {
    let current_dir_string = resource_path(valid, input, "");
    let current_dir = Path::new(&current_dir_string);

    transpile_dir(&current_dir, Some(&format!("{}.mamba", file_name)), Some(output_path), arguments)
        .map_err(|errs| OutTestErr(errs))?;

    // Check that reference check is proper Python file
    let cmd1 = Command::new(PYTHON)
        .arg("-m")
        .arg("py_compile")
        .arg(&resource_path(valid, input, &format!("{}_check.py", file_name)))
        .output()
        .expect("Could not run Python command.");

    // Check that output proper Python file
    let cmd2 = Command::new(PYTHON)
        .arg("-m")
        .arg("py_compile")
        .arg(&output_file)
        .output()
        .expect("Could not run Python command.");

    let check_src = resource_content(true, input, &format!("{}_check.py", file_name));
    // Replace CRLF with LF line endings
    let check_ast = python_src_to_stmts(&check_src.replace("\r\n", "\n"));

    let out_src = resource_content_path(output_file);
    let out_ast = python_src_to_stmts(&out_src);

    let width = 3;
    if cmd1.status.code().unwrap() != 0 {
        let msg = format!(
            "{}\nRunning Python command on reference resource: {}\n\
        Source:\n\
        ----------\n\
        {}\n\
        ----------",
            String::from_utf8(cmd1.stderr).unwrap().trim(),
            resource_path(valid, input, &format!("{}_check.py", file_name)),
            check_src
                .lines()
                .enumerate()
                .map(|(line, src)| { format!("{:width$} |   {}\n", line + 1, src) })
                .collect::<String>()
        );

        Err(OutTestErr(vec![msg]))
    } else if cmd2.status.code().unwrap() != 0 {
        let msg = format!(
            "{}Running Python command on Mamba output.\n\
        Source:\n\
        ----------\n\
        {}\n\
        ----------",
            String::from_utf8(cmd2.stderr).unwrap().trim(),
            out_src
                .lines()
                .enumerate()
                .map(|(line, src)| { format!("{:width$} |   {}\n", line + 1, src) })
                .collect::<String>()
        );

        Err(OutTestErr(vec![msg]))
    } else {
        Ok((check_ast, check_src, out_ast, out_src))
    }
}
