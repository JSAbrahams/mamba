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
        "Constructed context with {} classes, {} functions, {} fields",
        ctx.classes.len(),
        ctx.functions.len(),
        ctx.fields.len()
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

#[cfg(test)]
mod tests {
    use crate::check::ast::NodeTy;
    use crate::check::check_all;
    use crate::check::name::Name;
    use crate::parse::parse;

    #[test]
    fn if_stmt_no_type() {
        let src = "if True then 10 else 20";
        let ast = parse(src).unwrap();
        let result = check_all(&[*ast]).unwrap();

        assert_eq!(result.len(), 1);

        let if_stmt = result[0].clone();
        assert!(if_stmt.ty.is_none())
    }

    #[test]
    fn it_stmt_as_expression() {
        let src = "def a := if True then 10 else 20";
        let ast = parse(src).unwrap();
        let result = check_all(&[*ast]).unwrap();

        let statements = if let NodeTy::Block { statements } = &result[0].node {
            statements.clone()
        } else {
            panic!()
        };

        let expr = match &statements[0].node {
            NodeTy::VariableDef { expr, .. } => expr.clone().unwrap(),
            other => panic!("Expected variabledef: {:?}", other)
        };

        assert_eq!(expr.ty, Some(Name::from("Int")));
    }
}
