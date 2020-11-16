use crate::check::constrain::constraint::expected::Expect::{Access, Collection, Expression,
                                                            Function, Nullable, Tuple, Type};
use crate::check::constrain::constraint::iterator::Constraints;
use crate::check::constrain::constraint::Constraint;
use crate::check::constrain::unify::expression::unify_expression;
use crate::check::constrain::unify::function::unify_function;
use crate::check::constrain::unify::ty::unify_type;
use crate::check::constrain::Unified;
use crate::check::context::Context;

/// Unifies all constraints.
///
/// We use a mutable reference to constraints for performance reasons.
/// Otherwise, we have to make a entirely new copy of the list of all
/// constraints each time we do a recursive call to unify link.
pub fn unify_link(constraints: &mut Constraints, ctx: &Context, total: usize) -> Unified {
    if let Some(constraint) = &constraints.pop_constr() {
        let (left, right) = (&constraint.parent, &constraint.child);

        let pos = format!("({}={}) ", left.pos.start, right.pos.start);
        let count = total - constraints.len();
        let unify = format!("[{}\\{}] ", count, total);
        let msg = if constraint.msg.is_empty() {
            String::new()
        } else {
            format!("{{{}}} # ", constraint.msg)
        };
        trace!("{:width$}{}{}{}", pos, unify, msg, constraint, width = 15);

        match (&left.expect, &right.expect) {
            // trivially equal
            (left, right) if left == right => unify_link(constraints, ctx, total),

            (Function { .. }, Type { .. }) | (Access { .. }, _) =>
                unify_function(constraint, &left, &right, constraints, ctx, total),
            (Type { .. }, Function { .. }) | (_, Access { .. }) =>
                unify_function(constraint, &right, &left, constraints, ctx, total),

            (Expression { .. }, _) =>
                unify_expression(constraint, &left, &right, constraints, ctx, total),
            (_, Expression { .. }) =>
                unify_expression(constraint, &right, &left, constraints, ctx, total),

            (Type { .. }, _) | (_, Nullable) => unify_type(&left, &right, constraints, ctx, total),
            (_, Type { .. }) => unify_type(&right, &left, constraints, ctx, total),
            (Collection { .. }, Collection { .. }) =>
                unify_type(&right, &left, constraints, ctx, total),
            (Tuple { .. }, Tuple { .. }) => unify_type(&right, &left, constraints, ctx, total),

            _ => {
                let mut constr = reinsert(constraints, &constraint, total)?;
                unify_link(&mut constr, ctx, total + 1)
            }
        }
    } else {
        Ok(constraints.clone())
    }
}

/// Reinsert constraint.
///
/// The amount of attempts is a counter which states how often we allow
/// reinserts.
pub fn reinsert(constr: &mut Constraints, constraint: &Constraint, total: usize) -> Unified {
    let pos = format!("({}={}) ", constraint.parent.pos.start, constraint.child.pos.start);
    let count = format!("[reinserting {}\\{}] ", total - constr.len(), total);
    trace!("{:width$}{}{}", pos, count, constraint, width = 17);

    constr.reinsert(&constraint)?;
    Ok(constr.clone())
}
