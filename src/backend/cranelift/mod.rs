use std::fs::create_dir;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use cranelift_codegen::isa::{self, OwnedTargetIsa};
use cranelift_codegen::settings::{self};
use cranelift_module::default_libcall_names;
use cranelift_object::{ObjectBuilder, ObjectModule};
use log::{info, trace};
use target_lexicon::Triple;

use crate::backend::cranelift::result::{BackendErr, BackendResult};
use crate::check::ast::ASTTy;
use crate::common::position::Position;
use crate::common::result::WithSource;
use crate::{check_sources, strip_source_paths, Context};

mod convert;
mod link;
mod primitive;

pub mod result;

const TARGET: &str = "a.out";

/// Compile `source` to native object code and link it into an executable, written to disk.
///
/// Output is written to `target` (relative to `dir`) if given, otherwise to `a.out` in `dir`.
/// `triple`, if given, is a target triple passed on to Cranelift; if `None`, the host triple is
/// used.
pub fn write_output(
    dir: &Path,
    target: Option<&str>,
    source: &[(String, Option<PathBuf>)],
    src_path: &Path,
    triple: Option<&str>,
) -> Result<PathBuf, Vec<String>> {
    let out_file = dir.join(target.unwrap_or(TARGET));
    if let Some(parent) = out_file.parent() {
        if !parent.exists() {
            create_dir(parent).map_err(|e| vec![e.to_string()])?;
        }
    }
    info!(
        "Output executable will be stored at '{}'",
        out_file.display()
    );

    let objects = mamba_to_object(source, &src_path.to_path_buf(), triple)?;

    let mut object_files = vec![];
    for object in &objects {
        let mut file = tempfile::Builder::new()
            .suffix(".o")
            .tempfile()
            .map_err(|e| vec![e.to_string()])?;
        std::io::Write::write_all(&mut file, object).map_err(|e| vec![e.to_string()])?;
        object_files.push(file.into_temp_path());
    }

    link::link(&object_files, &out_file).map_err(|error| vec![error])?;

    Ok(out_file)
}

/// Compile mamba source to native object files, one per source, via the Cranelift backend.
///
/// `target`, if given, is a target triple passed on to Cranelift; if `None`, the host triple is
/// used. A path can optionally be given per source for error messages.
fn mamba_to_object(
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
            compile(ast_ty, target, &ctx)
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

/// Compile a single checked Mamba file to the bytes of a native object file.
///
/// `target`, if given, is a target triple (e.g. `x86_64-unknown-linux-gnu`).
/// If `None`, the host triple is used.
pub fn compile(ast_ty: &ASTTy, target: Option<&str>, ctx: &Context) -> BackendResult<Vec<u8>> {
    let isa = build_isa(target)?;

    let builder = ObjectBuilder::new(isa, "mamba", default_libcall_names())
        .map_err(|e| BackendErr::new(ast_ty.pos, &e.to_string()))?;
    let mut module = ObjectModule::new(builder);

    convert::lower_program(ast_ty, ctx, &mut module)?;

    module
        .finish()
        .emit()
        .map_err(|e| BackendErr::new(ast_ty.pos, &e.to_string()))
}

/// Create target which is understood by cranelift.
/// If None, then default to host architecture as target.
fn build_isa(target: Option<&str>) -> BackendResult<OwnedTargetIsa> {
    let flag_builder = settings::builder();
    let flags = settings::Flags::new(flag_builder);

    match target {
        Some(target) => {
            let triple = Triple::from_str(target).map_err(|e| {
                BackendErr::new(
                    Position::invisible(),
                    &format!("Invalid target '{target}': {e}"),
                )
            })?;
            isa::lookup(triple)
                .map_err(|e| {
                    BackendErr::new(
                        Position::invisible(),
                        &format!("Unsupported target '{target}': {e}"),
                    )
                })?
                .finish(flags)
                .map_err(|e| BackendErr::new(Position::invisible(), &e.to_string()))
        }
        None => cranelift_native::builder()
            .map_err(|e| {
                BackendErr::new(
                    Position::invisible(),
                    &format!("Unsupported host target: {e}"),
                )
            })?
            .finish(flags)
            .map_err(|e| BackendErr::new(Position::invisible(), &e.to_string())),
    }
}
