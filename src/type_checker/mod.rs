use std::convert::TryFrom;
use std::path::PathBuf;

use crate::parser::ast::AST;
use crate::type_checker::constraints::constraints;
use crate::type_checker::context::Context;
use crate::type_checker::modify::modify;
use crate::type_checker::type_result::{TypeErr, TypeResults};

pub mod context;
pub mod environment;

pub mod infer_type;
pub mod type_name;

mod constraints;
mod infer;
mod modify;
mod util;

pub mod type_result;

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
    let inputs: Vec<CheckInput> = inputs
        .iter()
        .map(|(ast, source, path)| match modify(ast, &context) {
            Ok(ast) => Ok((ast, source.clone(), path.clone())),
            Err(err) => Err(err)
        })
        .collect::<Result<_, _>>()?;

    for (ast, source, path) in &inputs {
        constraints(ast, &context).map_err(|err| {
            err.into_iter().map(|err| err.into_with_source(source, path)).collect::<Vec<TypeErr>>()
        })?;
    }

    Ok(inputs)
}
