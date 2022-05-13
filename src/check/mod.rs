use std::convert::TryFrom;
use std::path::PathBuf;

use crate::check::constrain::constraints;
use crate::check::context::Context;
use crate::check::result::{TypeErr, TypeResults};
use crate::parse::ast::AST;

mod constrain;
mod ident;

pub mod context;
pub mod name;
pub mod result;

pub type CheckInput = (AST, Option<String>, Option<PathBuf>);

/// Checks whether a given [AST](mamba::parser::ast::AST) is well
/// typed according to the specification of the language.
///
/// Should never panic.
///
/// # Failures
///
/// Any ill-typed [AST](mamba::parser::ast::AST) results in a
/// failure.
pub fn check_all(inputs: &[CheckInput]) -> TypeResults {
    let context = Context::try_from(inputs)?.into_with_primitives()?.into_with_std_lib()?;
    trace!(
        "Constructed context with\n - {} classes\n - {} functions\n - {} fields",
        context.class_count(),
        context.function_count(),
        context.field_count()
    );

    for (ast, source, path) in inputs {
        constraints(ast, &context).map_err(|err| {
            err.into_iter().map(|err| err.into_with_source(source, path)).collect::<Vec<TypeErr>>()
        })?;
    }

    Ok(Vec::from(inputs))
}
