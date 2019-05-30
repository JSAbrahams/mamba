use crate::parser::ast::ASTNodePos;

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
pub fn check(input: &[ASTNodePos]) -> Result<&[ASTNodePos], String> { Ok(input) }
