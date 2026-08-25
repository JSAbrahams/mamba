use std::str::FromStr;

use cranelift_codegen::isa::{self, OwnedTargetIsa};
use cranelift_codegen::settings::{self};
use cranelift_module::default_libcall_names;
use cranelift_object::{ObjectBuilder, ObjectModule};
use target_lexicon::Triple;

use crate::backend::cranelift::result::{BackendErr, BackendResult};
use crate::check::ast::ASTTy;
use crate::common::position::Position;
use crate::Context;

mod lower;
mod types;

pub mod link;
pub mod result;

/// Compile a single checked Mamba file to the bytes of a native object file.
///
/// `target`, if given, is a target triple (e.g. `x86_64-unknown-linux-gnu`); if `None`, the host
/// triple is used.
pub fn compile(ast_ty: &ASTTy, ctx: &Context, target: Option<&str>) -> BackendResult<Vec<u8>> {
    let isa = build_isa(target)?;
    let builder = ObjectBuilder::new(isa, "mamba", default_libcall_names())
        .map_err(|e| BackendErr::new(ast_ty.pos, &e.to_string()))?;
    let mut module = ObjectModule::new(builder);

    lower::lower_program(ast_ty, ctx, &mut module)?;

    module
        .finish()
        .emit()
        .map_err(|e| BackendErr::new(ast_ty.pos, &e.to_string()))
}

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
