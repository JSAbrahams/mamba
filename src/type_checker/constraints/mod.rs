use crate::parser::ast::AST;
use crate::type_checker::constraints::cons::Constraints;
use crate::type_checker::constraints::generate::generate;
use crate::type_checker::constraints::unify::unify;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::type_result::TypeErr;

pub mod cons;

mod generate;
mod unify;

pub type Constrained<T = (Constraints, Environment)> = Result<T, Vec<TypeErr>>;
pub type Unified = Result<Constraints, Vec<TypeErr>>;

pub fn constraints(ast: &AST, ctx: &Context) -> Unified {
    let (constrained, _) = generate(ast, &Environment::default(), ctx, &Constraints::new())?;

    debug!("CONSTRAINTS");
    for constraint in &constrained.constraints {
        debug!(
            "{:width$} {:?} == {:?}",
            format!("({},{})", constraint.0.pos, constraint.1.pos),
            constraint.0.expect,
            constraint.1.expect,
            width = 30
        );
    }

    debug!("UNIFICATION");
    let unified = unify(&constrained, ctx)?;
    Ok(unified)
}
