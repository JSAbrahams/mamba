use crate::parser::ast::ASTNodePos;
use crate::type_checker::type_result::TypeResults;
use std::path::PathBuf;

pub mod type_node;
pub mod type_result;

pub type CheckInput = (ASTNodePos, Option<String>, Option<PathBuf>);

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
    Ok(inputs
        .iter()
        .map(|(node_pos, source, path)| (node_pos.clone(), source.clone(), path.clone()))
        .collect())
}
