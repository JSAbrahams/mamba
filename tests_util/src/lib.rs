use std::cmp::max;
use std::fmt::{Debug, Formatter};
use std::fs::{self, create_dir, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use itertools::{EitherOrBoth, Itertools};
use python_parser::ast::Statement;
use tempfile::tempdir_in;

use mamba::common::delimit::newline_delimited;
use mamba::{transpile_dir, Arguments};

#[cfg(target_os = "linux")]
pub static PYTHON: &str = "python3.10";
#[cfg(target_os = "macos")]
pub static PYTHON: &str = "python3";
#[cfg(target_os = "windows")]
pub static PYTHON: &str = "python";

/// Run a Python file with [PYTHON] and return its captured stdout.
///
/// Unlike [test_directory]/[fallable], which only diff the generated Python's *AST* against a
/// reference, this actually executes the file -- for asserting on runtime behavior (e.g. what a
/// program actually prints), not just structural equivalence to a reference.
pub fn run_python(path: &Path) -> Result<String, String> {
    let output = Command::new(PYTHON)
        .arg(path)
        .output()
        .map_err(|e| format!("Could not run '{PYTHON} {}': {e}", path.display()))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(format!(
            "'{PYTHON} {}' exited with an error:\n{}",
            path.display(),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

pub struct OutTestErr(Vec<String>);

pub type OutTestRet<T = ()> = Result<T, OutTestErr>;

impl From<Vec<String>> for OutTestErr {
    fn from(value: Vec<String>) -> Self {
        OutTestErr(value)
    }
}

impl Debug for OutTestErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.iter().map(|err| write!(f, "{}\n", err)).collect()
    }
}

/// Test directory with default set to annotate output.
pub fn test_directory(valid: bool, input: &[&str], output: &[&str], file_name: &str) -> OutTestRet {
    let args = Arguments {
        annotate: true,
        ..Arguments::default()
    };
    test_directory_args(valid, input, output, file_name, &args)
}

pub fn test_directory_args(
    valid: bool,
    input: &[&str],
    _: &[&str],
    file_name: &str,
    args: &Arguments,
) -> OutTestRet {
    let (output_path, output_file) =
        resource_content_randomize(true, input, &format!("{}.py", file_name));

    let res = fallable(valid, input, &output_path, &output_file, file_name, args);
    delete_dir(&output_path).map_err(|_| OutTestErr(vec![]))?;
    let (check_ast, check_src, out_ast, out_src) = res?;

    // Convert to newline delimited string for more readable diff
    let check_string = newline_delimited(check_ast.iter().map(|stmt| format!("{:?}", stmt)));
    let out_string = newline_delimited(out_ast.iter().map(|stmt| format!("{:?}", stmt)));

    let longest_line = out_src
        .lines()
        .max_by(|l1, l2| l1.len().cmp(&l2.len()))
        .unwrap_or("")
        .len();
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

    assert_eq!(out_string, check_string, "\n{}", msg);
    Ok(())
}

pub fn fallable(
    valid: bool,
    input: &[&str],
    output_path: &str,
    output_file: &str,
    file_name: &str,
    arguments: &Arguments,
) -> OutTestRet<(Vec<Statement>, String, Vec<Statement>, String)> {
    let current_dir_string = resource_path(valid, input, "");
    let current_dir = Path::new(&current_dir_string);

    transpile_dir(
        &current_dir,
        Some(&format!("{}.mamba", file_name)),
        Some(output_path),
        arguments,
    )
    .map_err(|errs| OutTestErr(errs))?;

    // Check that reference check is proper Python file
    let cmd1 = Command::new(PYTHON)
        .arg("-m")
        .arg("py_compile")
        .arg(&resource_path(valid, input, &format!("{file_name}.py",)))
        .output()
        .expect("Could not run Python command.");

    // Check that output proper Python file
    let cmd2 = Command::new(PYTHON)
        .arg("-m")
        .arg("py_compile")
        .arg(&output_file)
        .output()
        .expect("Could not run Python command.");

    let check_src = resource_content(true, input, &format!("{}.py", file_name))?;
    // Replace CRLF with LF line endings
    let check_ast = python_src_to_stmts(&check_src.replace("\r\n", "\n"));

    let out_src = resource_content_path(output_file)?;
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
            resource_path(valid, input, &format!("{file_name}.py")),
            check_src
                .lines()
                .enumerate()
                .map(|(line, src)| { format!("{:width$} |   {}\n", line + 1, src) })
                .collect::<String>()
        );

        Err(OutTestErr(vec![msg]))
    } else if cmd2.status.code().unwrap() != 0 {
        let msg = format!(
            "{}\nRunning Python command on Mamba output.\n\
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

/// Get contents of file of given path as string.
pub fn resource_content_path(path: &str) -> Result<String, Vec<String>> {
    match File::open(path) {
        Ok(mut path) => {
            let mut content = String::new();
            match path.read_to_string(&mut content) {
                Ok(_) => Ok(content),
                Err(err) => Err(vec![format!("Error while reading file contents: {err}.")]),
            }
        }
        Err(err) => Err(vec![format!(
            "Error while opening file {path} while reading resource contents: {err}."
        )]),
    }
}

/// Get the path of a file at a given location.
///
/// * `valid` - Whether this is a happy or a sad path. See how test resources are structured.
/// * `subdirs` - Path to directory of resource under test.
/// * `file` - Name of file under test.
///
/// Returns:
/// - The absolute path of the resource, or the directory, as a string.
/// - The absolute path of the random output directory or file, to be deleted after the test.
pub fn resource_content_randomize(valid: bool, subdirs: &[&str], file: &str) -> (String, String) {
    let mut source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..") // outer mamba crate
        .join("tests")
        .join("resource")
        .join(if valid { "valid" } else { "invalid" });
    for dir in subdirs {
        source_path = source_path.join(dir);
    }

    if !source_path.exists() {
        create_dir(&source_path)
            .expect(format!("Path not found: {}", source_path.as_path().display()).as_str());
    }

    let source_path = tempdir_in(source_path.clone())
        .expect(format!("Could not create temp dir: {}", source_path.display()).as_str());
    let source = source_path.path();

    if file.is_empty() {
        (
            String::from(source.to_string_lossy()),
            String::from(source.to_string_lossy()),
        )
    } else {
        (
            String::from(source.to_string_lossy()),
            String::from(source.join(file).to_string_lossy()),
        )
    }
}

pub fn resource_content(valid: bool, subdirs: &[&str], file: &str) -> Result<String, Vec<String>> {
    resource_content_path(&resource_path(valid, subdirs, file))
}

pub fn resource_path(valid: bool, subdirs: &[&str], file: &str) -> String {
    let mut source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..") // outer mamba crate
        .join("tests")
        .join("resource")
        .join(if valid { "valid" } else { "invalid" });
    for dir in subdirs {
        source_path = source_path.join(dir);
    }

    if !source_path.exists() {
        create_dir(&source_path)
            .expect(format!("Path not found: {}", source_path.as_path().display()).as_str());
    }

    source_path = source_path.join(file);
    String::from(source_path.to_string_lossy())
}

pub fn delete_dir(resource_path: &String) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(&resource_path);
    if !path.exists() {
        Err(format!("{} does not exist", path.display()).into())
    } else {
        match fs::remove_dir_all(path) {
            Ok(_) => Ok(()),
            Err(err) => Err(format!("[{}] {}", err, path.display()).into()),
        }
    }
}

pub fn python_src_to_stmts(python_src: &String) -> Vec<Statement> {
    python_parser::file_input(python_parser::make_strspan(python_src.as_ref()))
        .unwrap()
        .1
}
