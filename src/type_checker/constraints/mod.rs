use crate::parser::ast::AST;
use crate::type_checker::checker_result::TypeErr;
use crate::type_checker::constraints::constraint::builder::ConstrBuilder;
use crate::type_checker::constraints::constraint::iterator::Constraints;
use crate::type_checker::constraints::generate::generate;
use crate::type_checker::constraints::unify::unify;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;

pub mod constraint;

mod generate;
mod unify;

pub type Constrained<T = (ConstrBuilder, Environment)> = Result<T, Vec<TypeErr>>;
pub type Unified<T = Constraints> = Result<T, Vec<TypeErr>>;

pub fn constraints(ast: &AST, ctx: &Context) -> Unified<Vec<Constraints>> {
    let (constrained, _) = generate(ast, &Environment::default(), ctx, &mut ConstrBuilder::new())?;
    let unified = unify(&constrained.all_constr(), ctx)?;
    Ok(unified)
}
