use crate::type_checker::constraints::constraint::iterator::Constraints;
use crate::type_checker::constraints::unify::unify_link::unify_link;
use crate::type_checker::constraints::Unified;
use crate::type_checker::context::Context;

mod substitute;
mod unify_link;

pub fn unify(all_constraints: &[Constraints], ctx: &Context) -> Unified<Vec<Constraints>> {
    let mut count = 1;
    all_constraints
        .iter()
        .map(|constraints| {
            println!("unifying set {}\\{}", count, all_constraints.len());
            count += 1;
            unify_link(&mut constraints.clone(), &Constraints::default(), ctx, constraints.len())
        })
        .collect()
}
