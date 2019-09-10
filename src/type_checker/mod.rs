use std::convert::TryFrom;
use std::path::PathBuf;

use crate::parser::ast::AST;
use crate::type_checker::context::environment::Environment;
use crate::type_checker::context::Context;
use crate::type_checker::infer::check;
use crate::type_checker::type_result::TypeResults;

mod context;
mod infer;
mod types;

pub mod type_result;

pub type CheckInput = (AST, Option<String>, Option<PathBuf>);

/// Checks whether a given [ASTNodePos](crate::parser::ast::ASTNodePos) is well
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
/// Any ill-typed [ASTNodePos](crate::parser::ast::ASTNodePos) results in a
/// failure.
///
/// // failure examples here
pub fn check_all(inputs: &[CheckInput]) -> TypeResults {
    // TODO use context during type checking and type inference stage
    let ctx = Context::try_from(inputs)?;
    let env = Environment::try_from(inputs)?;
    check(inputs, env, &ctx)?;

    Ok(inputs
        .iter()
        .map(|(node_pos, source, path)| (node_pos.clone(), source.clone(), path.clone()))
        .collect())
}
