use std::convert::TryFrom;
use std::path::PathBuf;

use crate::parser::ast::AST;
use crate::type_checker::context::Context;
use crate::type_checker::infer::infer_all;
use crate::type_checker::modify::modify;
use crate::type_checker::type_result::TypeResults;

pub mod context;
pub mod environment;

mod infer;
mod modify;
mod util;

pub mod type_result;
pub type CheckInput = (AST, Option<String>, Option<PathBuf>);

// TODO make type checker modify AST where necessary for more advanced language
// features

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
    let context = Context::try_from(inputs)?.into_with_primitives()?.into_with_std_lib()?;
    let inputs: Vec<CheckInput> = inputs
        .iter()
        .map(|(ast, source, path)| match modify(ast, &context) {
            Ok(ast) => Ok((ast, source.clone(), path.clone())),
            Err(err) => Err(err)
        })
        .collect::<Result<_, _>>()?;

    infer_all(&inputs, &context)?;
    Ok(inputs)
}
