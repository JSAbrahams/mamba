use std::convert::TryFrom;
use std::path::{Path, PathBuf};

use log::{info, trace};

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
        Backend::Python => backend::python::write_output(
            dir,
            target,
            &relative_paths,
            source_option_pairs.as_slice(),
            &src_path,
            &PipelineArguments::from(arguments),
        ),
        Backend::Bin { target: triple } => backend::cranelift::write_output(
            dir,
            target,
            source_option_pairs.as_slice(),
            &src_path,
            triple.as_deref(),
        ),
        Backend::Asm { target: triple } => {
            backend::cranelift::print_asm(
                source_option_pairs.as_slice(),
                &src_path,
                triple.as_deref(),
            )?;
            Ok(dir.to_path_buf())
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
pub(crate) fn strip_source_paths(
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
pub(crate) fn check_sources(
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

#[cfg(test)]
mod test {
    use std::path::Path;

    use crate::{transpile_dir, Arguments};

    #[test]
    fn transpile_dir_reports_missing_source_directory() {
        let dir = Path::new("/does/not/exist/anywhere");
        let result = transpile_dir(dir, None, None, &Arguments::default());

        let errs = result.unwrap_err();
        assert_eq!(errs.len(), 1);
        assert!(errs[0].contains("Source directory does not exist"));
    }
}
