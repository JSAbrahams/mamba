use crate::parser::ast::AST;
use crate::type_checker::constraints::cons::Constraints;
use crate::type_checker::constraints::generate::generate;
use crate::type_checker::constraints::unify::unify;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::type_result::TypeErr;

mod cons;
mod generate;
mod unify;

pub type Constrained = Result<(Constraints, Environment), Vec<TypeErr>>;
pub type Unified = Result<Constraints, Vec<TypeErr>>;

pub fn constraints(ast: &AST, env: &Environment, ctx: &Context) -> Unified {
    let (constrained, _) = generate(ast, env, ctx, &Constraints::new())?;
    unify(&constrained)
}
