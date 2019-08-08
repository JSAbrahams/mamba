use std::path::Path;
use std::path::PathBuf;

use leg::*;

use crate::core::to_source;
use crate::desugar::desugar;
use crate::lexer::tokenize;
use crate::parser::ast::ASTNodePos;
use crate::parser::parse;
use crate::pipeline::error::syntax_err;
use crate::pipeline::error::unimplemented_err;
use crate::type_checker::check;

mod error;
mod io;

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
pub fn transpile_directory(
    current_dir: &Path,
    maybe_in: Option<&str>,
    maybe_out: Option<&str>
) -> Result<PathBuf, Vec<(String, String)>> {
    let src_path = maybe_in.map_or(current_dir.join(IN_FILE), |p| current_dir.join(p));
    let out_dir = current_dir.join(maybe_out.unwrap_or(OUT_FILE));
    info(format!("Output will be stored in '{}'", out_dir.display()).as_str(), None, None);

    let relative_paths = io::relative_files(src_path.as_path()).map_err(|error| vec![error])?;
    let in_absolute_paths = if src_path.is_dir() {
        relative_paths.iter().map(|os_string| src_path.join(os_string)).collect()
    } else {
        vec![src_path]
    };
    let out_absolute_paths: Vec<PathBuf> =
        relative_paths.iter().map(|os_string| out_dir.join(os_string)).collect();

    let mut sources = vec![];
    for source_path in in_absolute_paths.clone() {
        let source = io::read_source(&source_path).map_err(|error| vec![error])?;
        sources.push(source);
    }

    let source_pairs = sources.iter().zip(in_absolute_paths.iter());
    let source_option_pairs: Vec<_> =
        source_pairs.map(|(source, path)| (source.clone(), Some(path.clone()))).collect();
    let mamba_source = mamba_to_python(source_option_pairs.as_slice())?;

    for (source, out_path) in mamba_source.iter().zip(out_absolute_paths) {
        let out_path = out_path.with_extension("py");
        io::write_source(source, &out_path).map_err(|error| vec![error])?;
    }

    Ok(out_dir)
}

/// Convert mamba source to python source.
///
/// For each mamba source, a path can optionally be given for display in error
/// messages. This path is not necessary however.
pub fn mamba_to_python(
    source: &[(String, Option<PathBuf>)]
) -> Result<Vec<String>, Vec<(String, String)>> {
    let mut ast_node_pairs = vec![];
    let mut syntax_errors = vec![];

    for (source, source_path) in source {
        match source_to_ast(source, source_path) {
            Ok(ast_node_pos) => ast_node_pairs.push((ast_node_pos, source, source_path)),
            Err(error) => syntax_errors.push(error)
        }
    }
    if !syntax_errors.is_empty() {
        return Err(syntax_errors);
    }

    let ast_nodes: Vec<_> = ast_node_pairs.iter().map(|(ast_node, ..)| ast_node.clone()).collect();
    check(ast_nodes.as_slice()).map_err(|errors| {
        let errors: Vec<(String, String)> =
            errors.iter().map(|error| (String::from("type"), error.clone())).collect();
        errors
    })?;

    // In future, desugaring stage should take in a vector and return a vector for
    // more complex desugaring rules.
    let mut python_sources = vec![];
    let mut type_errors = vec![];
    for (ast_node_pos, source, source_path) in ast_node_pairs {
        match ast_to_py(&ast_node_pos, source, source_path) {
            Ok(python_source) => python_sources.push(python_source),
            Err(error) => type_errors.push(error)
        }
    }

    if !type_errors.is_empty() {
        return Err(type_errors);
    }

    Ok(python_sources)
}

fn source_to_ast(
    source: &str,
    source_path: &Option<PathBuf>
) -> Result<ASTNodePos, (String, String)> {
    let tokens = tokenize(source).map_err(|err| (String::from("token"), err))?;
    parse(tokens.as_ref())
        .map_err(|err| (String::from("syntax"), syntax_err(&err, source, source_path)))
        .map(|ok| *ok)
}

fn ast_to_py(
    ast_node_pos: &ASTNodePos,
    source: &str,
    source_path: &Option<PathBuf>
) -> Result<String, (String, String)> {
    let desugared = desugar(ast_node_pos).map_err(|err| {
        (String::from("unimplemented"), unimplemented_err(&err, source, source_path))
    })?;
    Ok(to_source(&desugared))
}
