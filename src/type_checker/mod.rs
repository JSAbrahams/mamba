use std::convert::TryFrom;
use std::path::PathBuf;

use crate::parser::ast::AST;
use crate::type_checker::context::Context;
use crate::type_checker::type_result::TypeResults;

mod context;
mod environment;
mod infer;

pub mod type_result;
pub type CheckInput = (AST, Option<String>, Option<PathBuf>);

/// Checks whether a given [AST](crate::parser::ast::AST) is well
/// typed according to the specification of the language.
///
/// Should never panic.
///
/// # Examples
///
/// // examples here
///
/// # Failures
///
/// Any ill-typed [AST](crate::parser::ast::AST) results in a
/// failure.
///
/// // failure examples here
pub fn check_all(inputs: &[CheckInput]) -> TypeResults {
    Context::try_from(inputs)?;

    Ok(inputs
        .iter()
        .map(|(node_pos, source, path)| (node_pos.clone(), source.clone(), path.clone()))
        .collect())
}
