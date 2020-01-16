use crate::type_checker::constraints::cons::Constraints;
use crate::type_checker::constraints::unify::unify_link::unify_link;
use crate::type_checker::constraints::Unified;
use crate::type_checker::context::Context;

mod unify_link;

pub fn unify(constr: &Constraints, ctx: &Context) -> Unified {
    unify_link(&mut constr.clone(), &Constraints::new(), ctx)
}
