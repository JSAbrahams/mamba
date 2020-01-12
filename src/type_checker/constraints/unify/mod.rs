use crate::type_checker::constraints::cons::Constraints;
use crate::type_checker::constraints::Unified;
use crate::type_checker::environment::Environment;

pub fn unify(constr: &Constraints, _: &Environment) -> Unified { Ok(constr.clone()) }
