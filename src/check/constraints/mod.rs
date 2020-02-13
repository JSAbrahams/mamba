use crate::check::checker_result::TypeErr;
use crate::check::constraints::constraint::builder::ConstrBuilder;
use crate::check::constraints::constraint::iterator::Constraints;
use crate::check::constraints::generate::generate;
use crate::check::constraints::unify::unify;
use crate::check::context::Context;
use crate::check::environment::Environment;
use crate::parse::ast::AST;

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
