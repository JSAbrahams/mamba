extern crate ansi_term;
extern crate core;
#[macro_use]
extern crate log;
extern crate loggerv;

use std::convert::TryFrom;
use std::fs::create_dir;
use std::ops::Deref;
use std::path::{Path, PathBuf};

use crate::check::ast::ASTTy;
use crate::check::check;
use crate::check::context::Context;
use crate::check::result::TypeErr;
use crate::common::result::WithSource;
use crate::generate::{gen_arguments, GenArguments};
use crate::parse::ast::AST;
use crate::parse::parse;

pub mod common;

pub mod check;
pub mod generate;
pub mod parse;

pub mod io;

const TARGET: &str = "target";
const SOURCE: &str = "src";

#[derive(Default)]
pub struct Arguments {
    pub annotate: bool,
}

#[cfg(test)]
mod test_util {
    // Manual include, otherwise, we have to make this part of the interface to make use of these
    // utility functions in tests. We don't want this to be part of the interface.
    include!("../tests/common.rs");
}

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
pub fn transpile_dir(
    dir: &Path,
    src: Option<&str>,
    target: Option<&str>,
    arguments: &Arguments,
) -> Result<PathBuf, Vec<String>> {
    let src_path = src.map_or(dir.join(SOURCE), |p| dir.join(p));
    if !src_path.is_file() && !src_path.is_dir() {
        let msg = format!(
            "Source directory does not exist: {}",
            src_path.as_os_str().to_str().unwrap()
        );
        return Err(vec![msg]);
    } else if src_path.is_file() && !src_path.exists() {
        let msg = format!(
            "Source file does not exist: {}",
            src_path.as_os_str().to_str().unwrap()
        );
        return Err(vec![msg]);
    }

    let out_dir = dir.join(target.unwrap_or(TARGET));
    if !out_dir.exists() {
        create_dir(&out_dir).map_err(|e| vec![e.to_string()])?;
    }
    info!("Input is '{}'", src_path.display());
    info!("Output will be stored in '{}'", out_dir.display());

    let relative_paths = io::relative_files(src_path.as_path()).map_err(|error| vec![error])?;
    let in_absolute_paths = if src_path.is_dir() {
        relative_paths
            .iter()
            .map(|os_string| src_path.join(os_string))
            .collect()
    } else {
        vec![src_path.clone()]
    };
    let out_absolute_paths: Vec<PathBuf> = relative_paths
        .iter()
        .map(|os_string| out_dir.join(os_string))
        .collect();

    info!(
        "Transpiling {} file {}",
        out_absolute_paths.len(),
        if out_absolute_paths.len() > 1 {
            "s"
        } else {
            ""
        }
    );

    let mut sources = vec![];
    for source_path in in_absolute_paths.clone() {
        let source = io::read_source(&source_path).map_err(|error| vec![error])?;
        sources.push(source);
    }

    let source_pairs = sources.iter().zip(in_absolute_paths.iter());
    let source_option_pairs: Vec<_> = source_pairs
        .map(|(source, path)| (source.clone(), Some(path.clone())))
        .collect();

    let pipeline_arg = PipelineArguments::from(arguments);
    let mamba_source = mamba_to_python(source_option_pairs.as_slice(), &src_path, &pipeline_arg)?;

    for (source, out_path) in mamba_source.iter().zip(out_absolute_paths) {
        let out_path = out_path.with_extension("py");
        io::write_source(source, &out_path).map_err(|error| vec![error])?;
    }

    Ok(out_dir)
}

pub struct PipelineArguments {
    pub annotate: bool,
}

impl From<&Arguments> for PipelineArguments {
    fn from(arguments: &Arguments) -> Self {
        PipelineArguments {
            annotate: arguments.annotate,
        }
    }
}

/// Convert mamba source to python source.
///
/// For each mamba source, a path can optionally be given for display in error
/// messages. This path is not necessary however.
pub fn mamba_to_python(
    source: &[(String, Option<PathBuf>)],
    source_dir: &PathBuf,
    pipeline_args: &PipelineArguments,
) -> Result<Vec<String>, Vec<String>> {
    // Strip until source
    let strip_prefix = |p: PathBuf| {
        p.strip_prefix(source_dir)
            .map(|p| {
                PathBuf::from(&source_dir.iter().last().unwrap_or_else(|| "".as_ref())).join(p)
            })
            .unwrap_or(p)
    };
    let source: Vec<(String, Option<PathBuf>)> = source
        .iter()
        .map(|(src, dir)| (src.clone(), dir.clone().map(strip_prefix)))
        .collect();

    let (asts, parse_errs): (Vec<_>, Vec<_>) = source
        .iter()
        .map(|(src, path)| {
            parse(src)
                .map_err(|err| err.with_source(&Some(src.clone()), &path.clone()))
                .map(|ok| ok.deref().clone())
        })
        .partition(Result::is_ok);

    let parse_errs: Vec<_> = parse_errs.into_iter().map(Result::unwrap_err).collect();
    if !parse_errs.is_empty() {
        return Err(parse_errs.iter().map(|err| format!("{err}")).collect());
    }

    let asts: Vec<AST> = asts.into_iter().map(Result::unwrap).collect();
    trace!("Parsed {} files", asts.len());

    let ctx = Context::try_from(asts.as_ref())
        .map_err(|errs| errs.iter().map(|e| format!("{e}")).collect::<Vec<String>>())?;
    let (typed_ast, type_errs): (Vec<_>, Vec<_>) = asts
        .iter()
        .zip(&source)
        .map(|(ast, (src, path))| {
            check(ast, &ctx).map_err(|errs| {
                errs.iter()
                    .map(|err| err.clone().with_source(&Some(src.clone()), &path.clone()))
                    .collect()
            })
        })
        .partition(Result::is_ok);

    let type_errs: Vec<Vec<TypeErr>> = type_errs.into_iter().map(Result::unwrap_err).collect();
    if !type_errs.is_empty() {
        return Err(type_errs
            .iter()
            .flatten()
            .map(|err| format!("{err}"))
            .collect());
    }
    let typed_ast = typed_ast
        .into_iter()
        .map(Result::unwrap)
        .collect::<Vec<ASTTy>>();

    trace!("Checked {} files", typed_ast.len());

    let gen_args = GenArguments::from(pipeline_args);
    let (py_sources, gen_errs): (Vec<_>, Vec<_>) = typed_ast
        .iter()
        .zip(&source)
        .map(|(ast_ty, (src, path))| {
            gen_arguments(ast_ty, &gen_args, &ctx)
                .map_err(|err| err.with_source(&Some(src.clone()), &path.clone()))
                .map(|core| format!("{core}"))
        })
        .partition(Result::is_ok);

    let gen_errs: Vec<_> = gen_errs.into_iter().map(Result::unwrap_err).collect();
    if !gen_errs.is_empty() {
        return Err(gen_errs.iter().map(|err| format!("{err}")).collect());
    }

    let py_sources: Vec<String> = py_sources.into_iter().map(Result::unwrap).collect();
    trace!("Converted {} files to Python source", py_sources.len());

    Ok(py_sources)
}
