use crate::core::to_source;
use crate::desugar::desugar;
use crate::lexer::tokenize;
use crate::parser::ast::ASTNodePos;
use crate::parser::parse;
use crate::pipeline::error::syntax_err;
use crate::type_checker::check;
use glob::glob;
use leg::*;
use pathdiff::diff_paths;
use std::ffi::OsString;
use std::fs;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

mod error;

const OUT_FILE: &str = "target";
const IN_FILE: &str = "src";

/// Convert `*.mamba` files to `*.py`.
///
/// For input, the rules are as follows:
/// If file, file taken as input.
/// If directory, recursively search all sub-directories for mamba files.
/// If no input given, current directory taken is directory.
///
/// For output, the rules are as follows:
/// Output directory to story mamba files.
/// Output directory structure reflects input directory structure.
/// If no output given, target directory created in current directory and output
/// stored here.
pub fn mamba_to_python(
    current_dir: &Path,
    maybe_in: Option<&str>,
    maybe_out: Option<&str>
) -> Result<PathBuf, String> {
    let in_path = maybe_in.map_or(current_dir.join(IN_FILE), |p| current_dir.join(p));
    let out_dir = current_dir.join(maybe_out.unwrap_or(OUT_FILE));

    info(format!("Output will be stored in '{}'", out_dir.display()).as_str(), None, None);

    let relative_file_paths = if in_path.is_dir() {
        relative_paths(in_path.as_path())?
    } else {
        let in_file_name = in_path.file_name().unwrap_or_else(|| unreachable!());
        vec![in_file_name.to_os_string()]
    };

    match pipeline(current_dir, out_dir.as_path(), relative_file_paths.as_slice()) {
        Ok(..) => {
            success(format!("Output stored in: '{}'", out_dir.display()).as_str(), None, None);
            Ok(out_dir.to_owned())
        }
        Err(errors) => {
            for (error_type, error_message) in errors {
                error(error_message.as_str(), Some(error_type.as_str()), None)
            }
            Err(String::from("An error occurred"))
        }
    }
}

/// Get all `*.mamba` files paths relative to given directory.
fn relative_paths(in_dir: &Path) -> Result<Vec<OsString>, String> {
    let pattern_path = in_dir.to_owned().join("**").join("*.mamba");
    let pattern = pattern_path.as_os_str().to_string_lossy();
    let glob =
        glob(pattern.as_ref()).map_err(|e| format!("Unable to recursively find files: {}", e))?;

    let mut relative_paths = vec![];
    for absolute_result in glob {
        let absolute_path = absolute_result.map_err(|e| e.to_string())?;
        let relative_path = diff_paths(absolute_path.as_path(), in_dir).ok_or("")?;
        relative_paths.push(relative_path.into_os_string());
    }

    Ok(relative_paths)
}

pub fn pipeline(
    in_dir: &Path,
    out_dir: &Path,
    relative_file_paths: &[OsString]
) -> Result<(), Vec<(String, String)>> {
    let mut ast_trees = vec![];
    let mut syntax_errors = vec![];

    for in_path in relative_file_paths.iter().map(|p| in_dir.join(p)) {
        match input(&in_path) {
            Ok(ast_tree) => ast_trees.push(ast_tree),
            Err(err) => syntax_errors.push(err)
        }
    }

    if !syntax_errors.is_empty() {
        return Err(syntax_errors);
    }

    let type_err: fn(Vec<String>) -> Vec<(String, String)> =
        |e: Vec<String>| e.iter().map(|e| (String::from("typing"), e.clone())).collect();
    let typed_ast_trees = check(ast_trees.as_slice()).map_err(type_err)?;
    debug_assert_eq!(typed_ast_trees.len(), relative_file_paths.len());

    let mut i = 0;
    for typed_ast_tree in typed_ast_trees.iter() {
        match output(
            &typed_ast_tree,
            out_dir.join(relative_file_paths[i].clone()).with_extension("py").as_path()
        ) {
            Ok(..) => i += 1,
            Err(err) => return Err(vec![err])
        }
    }
    Ok(())
}

fn input(in_path: &PathBuf) -> Result<ASTNodePos, (String, String)> {
    let mut input_file = OpenOptions::new()
        .read(true)
        .open(in_path.clone())
        .map_err(|e| (String::from("input"), format!("{}: '{}'", e, in_path.display())))?;
    let mut source = String::new();
    input_file
        .read_to_string(&mut source)
        .map_err(|e| (String::from("input"), format!("{}: '{}'", e, in_path.display())))?;
    success(format!("'{}'", in_path.display()).as_str(), Some("input"), None);

    let tokens = tokenize(source.as_ref()).map_err(|e| (String::from("token"), e.to_string()))?;
    parse(&tokens)
        .map_err(|err| (String::from("syntax"), syntax_err(&err, &source, in_path)))
        .map(|ok| *ok)
}

fn output(typed_ast_tree: &ASTNodePos, out_path: &Path) -> Result<(), (String, String)> {
    let core_tree = desugar(&typed_ast_tree).unwrap();
    let source = to_source(&core_tree);

    match out_path.parent() {
        Some(parent) => fs::create_dir_all(parent)
            .map_err(|e| (String::from("output"), format!("{}: {}", e, out_path.display()))),
        None => Err((
            String::from("output"),
            format!("output had no parent directory: '{}'", out_path.display())
        ))
    }?;

    OpenOptions::new()
        .write(true)
        .create(true)
        .open(out_path)
        .map_err(|e| (String::from("output"), format!("{}: '{}'", e, out_path.display())))?
        .write_all(source.as_ref())
        .map_err(|e| (String::from("output"), format!("{}: `{}`", e, out_path.display())))?;

    success(format!("'{}'", out_path.display()).as_str(), Some("output"), None);
    Ok(())
}
