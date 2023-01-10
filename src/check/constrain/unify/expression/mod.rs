use crate::check::constrain::constraint::Constraint;
use crate::check::constrain::constraint::expected::Expect::Expression;
use crate::check::constrain::constraint::iterator::Constraints;
use crate::check::constrain::Unified;
use crate::check::constrain::unify::expression::substitute::substitute;
use crate::check::constrain::unify::finished::Finished;
use crate::check::constrain::unify::link::unify_link;
use crate::check::context::Context;

pub mod substitute;

pub fn unify_expression(constraint: &Constraint, constraints: &mut Constraints, finished: &mut Finished, ctx: &Context, count: usize, total: usize) -> Unified {
    let (left, right) = (&constraint.parent, &constraint.child);
    match (&left.expect, &right.expect) {
        (Expression { .. }, _) => substitute(constraints, right, left, count, total)?,
        _ => substitute(constraints, left, right, count, total)?
    }

    unify_link(constraints, finished, ctx, total)
}
