use std::convert::TryFrom;
use std::fs::create_dir;
use std::path::{Path, PathBuf};

use log::{info, trace};

use crate::backend::python::{gen_arguments, GenArguments};
use crate::backend::Backend;
use crate::check::ast::ASTTy;
use crate::check::check;
use crate::check::context::Context;
use crate::check::result::TypeErr;
use crate::common::result::WithSource;
use crate::parse::ast::AST;

pub mod common;

pub mod backend;
pub mod check;
pub mod parse;

pub mod cli;
pub mod io;

const TARGET: &str = "target";
const SOURCE: &str = "src";

#[derive(Default)]
pub struct Arguments {
    pub annotate: bool,
    pub backend: Backend,
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

    let relative_paths = io::relative_files(src_path.as_path()).map_err(|error| vec![error])?;
    let in_absolute_paths: Vec<PathBuf> = if src_path.is_dir() {
        relative_paths
            .iter()
            .map(|os_string| src_path.join(os_string))
            .collect()
    } else {
        vec![src_path.clone()]
    };

    info!(
        "Compiling {} file{}",
        in_absolute_paths.len(),
        if in_absolute_paths.len() > 1 { "s" } else { "" }
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

    match &arguments.backend {
        Backend::Python => {
            let out_dir = dir.join(target.unwrap_or(TARGET));
            if !out_dir.exists() {
                create_dir(&out_dir).map_err(|e| vec![e.to_string()])?;
            }
            info!("Output will be stored in '{}'", out_dir.display());

            let out_absolute_paths: Vec<PathBuf> = relative_paths
                .iter()
                .map(|os_string| out_dir.join(os_string))
                .collect();

            let pipeline_arg = PipelineArguments::from(arguments);
            let mamba_source =
                mamba_to_python(source_option_pairs.as_slice(), &src_path, &pipeline_arg)?;

            for (source, out_path) in mamba_source.iter().zip(out_absolute_paths) {
                let out_path = out_path.with_extension("py");
                io::write_source(source, &out_path).map_err(|error| vec![error])?;
            }

            Ok(out_dir)
        }
        Backend::Bin { target: triple } => {
            let out_file = dir.join(target.unwrap_or("a.out"));
            if let Some(parent) = out_file.parent() {
                if !parent.exists() {
                    create_dir(parent).map_err(|e| vec![e.to_string()])?;
                }
            }
            info!(
                "Output executable will be stored at '{}'",
                out_file.display()
            );

            let objects =
                mamba_to_object(source_option_pairs.as_slice(), &src_path, triple.as_deref())?;

            let mut object_files = vec![];
            for object in &objects {
                let mut file = tempfile::Builder::new()
                    .suffix(".o")
                    .tempfile()
                    .map_err(|e| vec![e.to_string()])?;
                std::io::Write::write_all(&mut file, object).map_err(|e| vec![e.to_string()])?;
                object_files.push(file.into_temp_path());
            }

            backend::cranelift::link::link(&object_files, &out_file)
                .map_err(|error| vec![error])?;

            Ok(out_file)
        }
    }
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

/// Strip each source's path down to be relative to `source_dir`, for nicer error messages.
fn strip_source_paths(
    source: &[(String, Option<PathBuf>)],
    source_dir: &PathBuf,
) -> Vec<(String, Option<PathBuf>)> {
    let strip_prefix = |p: PathBuf| {
        p.strip_prefix(source_dir)
            .map(|p| {
                PathBuf::from(&source_dir.iter().next_back().unwrap_or_else(|| "".as_ref())).join(p)
            })
            .unwrap_or(p)
    };
    source
        .iter()
        .map(|(src, dir)| (src.clone(), dir.clone().map(strip_prefix)))
        .collect()
}

/// Parse and type-check `source`, shared by every backend -- parsing and type-checking don't
/// depend on which backend eventually turns the result into output.
fn check_sources(
    source: &[(String, Option<PathBuf>)],
) -> Result<(Context, Vec<ASTTy>), Vec<String>> {
    let (asts, parse_errs): (Vec<_>, Vec<_>) = source
        .iter()
        .map(|(src, path)| {
            src.parse::<AST>()
                .map_err(|err| Box::new(err.with_source(&Some(src.clone()), &path.clone())))
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
        .zip(source)
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
    Ok((ctx, typed_ast))
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
    let source = strip_source_paths(source, source_dir);
    let (ctx, typed_ast) = check_sources(&source)?;

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

/// Compile mamba source to native object files, one per source, via the Cranelift backend.
///
/// `target`, if given, is a target triple passed on to Cranelift; if `None`, the host triple is
/// used. As with [`mamba_to_python`], a path can optionally be given per source for error
/// messages.
pub fn mamba_to_object(
    source: &[(String, Option<PathBuf>)],
    source_dir: &PathBuf,
    target: Option<&str>,
) -> Result<Vec<Vec<u8>>, Vec<String>> {
    let source = strip_source_paths(source, source_dir);
    let (ctx, typed_ast) = check_sources(&source)?;

    let (objects, gen_errs): (Vec<_>, Vec<_>) = typed_ast
        .iter()
        .zip(&source)
        .map(|(ast_ty, (src, path))| {
            backend::cranelift::compile(ast_ty, &ctx, target)
                .map_err(|err| err.with_source(&Some(src.clone()), &path.clone()))
        })
        .partition(Result::is_ok);

    let gen_errs: Vec<_> = gen_errs.into_iter().map(Result::unwrap_err).collect();
    if !gen_errs.is_empty() {
        return Err(gen_errs.iter().map(|err| format!("{err}")).collect());
    }

    trace!("Compiled {} files to object code", objects.len());
    Ok(objects.into_iter().map(Result::unwrap).collect())
}
