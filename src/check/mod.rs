use std::collections::HashMap;
use std::convert::TryFrom;

use crate::check::ast::ASTTy;
use crate::check::constrain::constraint::iterator::Constraints;
use crate::check::constrain::constraints;
use crate::check::context::Context;
use crate::check::name::Name;
use crate::check::result::TypeResult;
use crate::common::position::Position;
use crate::parse::ast::AST;
use crate::TypeErr;

mod constrain;
mod ident;

pub mod ast;
pub mod context;
pub mod name;
pub mod result;

/// Checks whether a given [AST](mamba::parser::ast::AST) is well
/// typed according to the specification of the language.
///
/// # Failures
///
/// Any ill-typed [AST](mamba::parser::ast::AST) results in a failure.
pub fn check(ast: &AST, ctx: &Context) -> TypeResult {
    trace!(
        "Constructed context with\n - {} classes\n - {} functions\n - {} fields",
        ctx.class_count(),
        ctx.function_count(),
        ctx.field_count()
    );

    constraints(ast, ctx).map(|all_constraints| {
        let pos_to_name: HashMap<Position, Name> = all_constraints
            .iter()
            .fold(Constraints::new(), |acc, constr| {
                constr.finished.iter().fold(acc, |mut acc, (pos, name)| {
                    acc.push_ty(*pos, name);
                    acc
                })
            }).finished;

        ASTTy::from((ast, &pos_to_name))
    })
}

pub fn check_all(asts: &[AST]) -> TypeResult<Vec<ASTTy>> {
    let ctx = Context::try_from(asts);

    match ctx {
        Ok(ctx) => {
            let (typed_ast, type_errs): (Vec<_>, Vec<_>) = asts
                .iter()
                .map(|ast| check(ast, &ctx))
                .partition(Result::is_ok);

            let type_errs: Vec<Vec<TypeErr>> = type_errs.into_iter().map(Result::unwrap_err).collect();
            if !type_errs.is_empty() {
                Err(type_errs.into_iter().flatten().collect())
            } else {
                Ok(typed_ast.into_iter().map(Result::unwrap).collect())
            }
        }
        Err(errs) => Err(errs),
    }
}
