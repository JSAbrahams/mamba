use std::ffi::OsString;
use std::fs::create_dir;
use std::path::{Path, PathBuf};

use log::{info, trace};

use crate::backend::python::ast::node::PythonCore;
use crate::backend::python::convert::convert_node;
use crate::backend::python::convert::state::{Imports, State};
use crate::backend::python::result::GenResult;
use crate::check::ast::ASTTy;
use crate::common::result::WithSource;
use crate::{check_sources, io, strip_source_paths, Context, PipelineArguments};

mod convert;

pub mod ast;
pub mod name;

pub mod result;

const TARGET: &str = "target";

/// Transpile `source` to Python and write the result to disk under `dir`.
///
/// Output is written to `target` (relative to `dir`) if given, otherwise to a `target`
/// directory created in `dir`; the output directory structure mirrors `relative_paths`.
pub fn write_output(
    dir: &Path,
    target: Option<&str>,
    relative_paths: &[OsString],
    source: &[(String, Option<PathBuf>)],
    src_path: &Path,
    pipeline_args: &PipelineArguments,
) -> Result<PathBuf, Vec<String>> {
    let out_dir = dir.join(target.unwrap_or(TARGET));
    if !out_dir.exists() {
        create_dir(&out_dir).map_err(|e| vec![e.to_string()])?;
    }
    info!("Output will be stored in '{}'", out_dir.display());

    let py_sources = mamba_to_python(source, &src_path.to_path_buf(), pipeline_args)?;

    for (source, relative_path) in py_sources.iter().zip(relative_paths) {
        let out_path = out_dir.join(relative_path).with_extension("py");
        io::write_source(source, &out_path).map_err(|error| vec![error])?;
    }

    Ok(out_dir)
}

/// Convert mamba source to python source.
///
/// For each mamba source, a path can optionally be given for display in error
/// messages. This path is not necessary however.
fn mamba_to_python(
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

#[derive(Default)]
pub struct GenArguments {
    pub annotate: bool,
}

impl From<&PipelineArguments> for GenArguments {
    fn from(pipeline_args: &PipelineArguments) -> Self {
        GenArguments {
            annotate: pipeline_args.annotate,
        }
    }
}

/// Consumes the given [AST](mamba::parse::ast::AST) and produces
/// a [PythonCore](mamba::backend::python::ast::node::PythonCore) node.
///
/// Note that the given [AST](mamba::parse::ast::AST) must be
/// correctly formed. Therefore, malformed
/// [AST](mamba::parse::ast::AST)'s should be caught by either
/// the parser or the type checker.
///
/// # Examples
///
/// ```
/// # use mamba::check::ast::ASTTy;
/// # use mamba::parse::ast::Node;
/// # use mamba::parse::ast::AST;
/// # use mamba::backend::python::ast::node::PythonCore;
/// # use mamba::common::position::{CaretPos, Position};
/// # use mamba::backend::python::gen;
/// let node = Node::ReturnEmpty;
/// let ast = AST::new(Position::new(CaretPos::new(1, 1), CaretPos::new(1, 5)), node);
/// let ast_ty = ASTTy::from(&ast);
/// let core_result = gen(&ast_ty).unwrap();
///
/// assert_eq!(core_result, PythonCore::Return { expr: Box::from(PythonCore::None) });
/// ```
///
/// # Failures
///
/// Fails if converting a construct which has not been implemented yet.
///
/// ```rust
/// # use mamba::check::ast::ASTTy;
/// # use mamba::parse::ast::Node;
/// # use mamba::parse::ast::AST;
/// # use mamba::backend::python::ast::node::PythonCore;
/// # use mamba::common::position::{CaretPos, Position};
/// # use mamba::backend::python::gen;
/// let cond_node = Node::Int { lit: String::from("56") };
/// let cond_pos = AST::new(Position::new(CaretPos::new(0, 0), CaretPos::new(0, 5)), cond_node);
/// let node = Node::Condition { cond: Box::from(cond_pos), el: None };
/// let ast = AST::new(Position::new(CaretPos::new(0, 0), CaretPos::new(0, 5)), node);
/// let ast_ty = ASTTy::from(&ast);
/// let core_result = gen(&ast_ty);
///
/// assert!(core_result.is_err());
/// ```
///
/// # Panics
///
/// A malformed [AST](crate::parser::ast::AST) causes this stage
/// to panic.
pub fn gen_arguments(ast_ty: &ASTTy, gen_args: &GenArguments, ctx: &Context) -> GenResult {
    let state = State::from(gen_args);

    let import = &mut Imports::new();
    match convert_node(ast_ty, import, &state, ctx)? {
        PythonCore::Block { statements } => Ok(PythonCore::Block {
            statements: import.imports().into_iter().chain(statements).collect(),
        }),
        other if !import.is_empty() => Ok(PythonCore::Block {
            statements: import.imports().into_iter().chain(vec![other]).collect(),
        }),
        other => Ok(other),
    }
}

pub fn gen(ast_ty: &ASTTy) -> GenResult {
    gen_arguments(ast_ty, &GenArguments::default(), &Context::default())
}
