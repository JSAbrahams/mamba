use crate::type_checker::constraints::cons::Constraints;
use crate::type_checker::constraints::Unified;

pub fn unify(constr: &Constraints) -> Unified { Ok(constr.clone()) }
