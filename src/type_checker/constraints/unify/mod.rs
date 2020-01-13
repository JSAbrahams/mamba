use crate::type_checker::constraints::cons::Constraints;
use crate::type_checker::constraints::unify::unify_link::unify_link;
use crate::type_checker::constraints::Unified;

mod unify_link;

pub fn unify(constr: &Constraints) -> Unified { unify_link(&constr.clone(), &Constraints::new()) }
