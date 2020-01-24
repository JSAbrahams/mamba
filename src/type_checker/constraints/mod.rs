use crate::parser::ast::AST;
use crate::type_checker::constraints::constraint::Constraints;
use crate::type_checker::constraints::generate::generate;
use crate::type_checker::constraints::unify::unify;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::type_result::TypeErr;

pub mod constraint;

mod generate;
mod unify;

pub type Constrained<T = (Constraints, Environment)> = Result<T, Vec<TypeErr>>;
pub type Unified = Result<Constraints, Vec<TypeErr>>;

pub fn constraints(ast: &AST, ctx: &Context) -> Unified {
    let (constrained, _) = generate(ast, &Environment::default(), ctx, &mut Constraints::new())?;

    trace!("CONSTRAINTS");
    for constraint in &constrained.constraints {
        trace!(
            "{:width$} {:?} == {:?}",
            format!("({},{})", constraint.left.pos, constraint.right.pos),
            constraint.left.expect,
            constraint.right.expect,
            width = 30
        );
    }

    trace!("UNIFICATION");
    let unified = unify(&constrained, ctx)?;
    Ok(unified)
}
