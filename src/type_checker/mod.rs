use crate::parser::ast::ASTNodePos;
use crate::type_checker::stage_1::Context;
use crate::type_checker::stage_2::type_check;
use std::clone::Clone;

mod stage_1;
mod stage_2;
mod util;

pub mod type_node;
pub mod type_result;

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
pub fn check(input: &[ASTNodePos]) -> Result<Vec<ASTNodePos>, Vec<String>> {
    let context = Context::new(input)?;
    let (_, errors): (Vec<_>, Vec<_>) = input
        .iter()
        .map(|node_pos| type_check(&context, node_pos.clone()))
        .partition(Result::is_ok);

    if errors.is_empty() {
        Ok(input.to_vec())
    } else {
        Err(errors.into_iter().map(Result::unwrap_err).collect())
    }
}
