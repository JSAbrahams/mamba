use log::trace;

use crate::check::constrain::constraint::expected::Expect::{Access, Expression, Function, Type};
use crate::check::constrain::constraint::iterator::Constraints;
use crate::check::constrain::constraint::Constraint;
use crate::check::constrain::unify::expression::sub;
use crate::check::constrain::unify::finished::Finished;
use crate::check::constrain::unify::function::unify_function;
use crate::check::constrain::unify::ty::unify_type;
use crate::check::constrain::Unified;
use crate::check::context::Context;
use crate::check::name::ContainsTemp;

/// Unify every constraint in the set, front to back, until none are left.
///
/// A loop, not recursion, so a large set cannot overflow the stack.
/// Each step may substitute into, push onto, or reinsert into `constraints`, see [`unify_constraint`].
/// `total` only feeds the `count\total` progress in the trace.
pub fn unify_link(
    constraints: &mut Constraints,
    finished: &mut Finished,
    ctx: &Context,
    total: usize,
) -> Unified<()> {
    let mut total = total;
    while let Some(constraint) = constraints.pop_constr() {
        total = unify_constraint(&constraint, constraints, finished, ctx, total)?;
    }
    Ok(())
}

/// Unify one constraint, recording any known type in `finished`.
/// Returns `total`, grown by however many constraints this step added, for the trace.
fn unify_constraint(
    constraint: &Constraint,
    constraints: &mut Constraints,
    finished: &mut Finished,
    ctx: &Context,
    total: usize,
) -> Unified<usize> {
    let (left, right) = (&constraint.parent, &constraint.child);
    let count = total.saturating_sub(constraints.len());
    trace_constraint(constraint, count, total);

    if let Type { name } = &left.expect {
        if !name.contains_temp() {
            finished.push_ty(ctx, right.pos, right, name)?;
        }
    }
    if let Type { name } = &right.expect {
        if !name.contains_temp() {
            finished.push_ty(ctx, left.pos, left, name)?;
        }
    }

    match (&left.expect, &right.expect) {
        (left, right) if left == right => Ok(total),

        (Function { .. }, Type { .. })
        | (Type { .. }, Function { .. })
        | (Access { .. }, _)
        | (_, Access { .. }) => unify_function(constraint, constraints, finished, ctx, total),

        (Expression { .. }, _) => {
            sub(constraints, right, left, count, total)?;
            Ok(total)
        }
        (_, Expression { .. }) => {
            sub(constraints, left, right, count, total)?;
            Ok(total)
        }

        (Type { .. }, _) | (_, Type { .. }) => unify_type(constraint, constraints, ctx, total),

        _ => {
            reinsert(constraints, constraint, total)?;
            Ok(total + 1)
        }
    }
}

/// Trace the constraint about to be unified, as `count\total` constraints seen.
fn trace_constraint(constraint: &Constraint, count: usize, total: usize) {
    let (left, right) = (&constraint.parent, &constraint.child);
    let pos = format!("{}={} ", left.pos, right.pos);
    let unify = format!("{count}\\{total}");
    let msg = if constraint.msg.is_empty() {
        String::new()
    } else {
        format!(" {}", constraint.msg)
    };
    trace!(
        "{:width$}[{}{}]  {}",
        pos,
        unify,
        msg,
        constraint,
        width = 27
    );
}

/// Reinsert constraint.
///
/// The amount of attempts is a counter which states how often we allow
/// reinserts.
pub fn reinsert(constr: &mut Constraints, constraint: &Constraint, total: usize) -> Unified<()> {
    let pos = format!(
        "({}={}) ",
        constraint.parent.pos.start, constraint.child.pos.start
    );
    let count = format!("[reinserting {}\\{}] ", total - constr.len(), total);
    trace!("{:width$}{}{}", pos, count, constraint, width = 17);

    constr.reinsert(constraint)?;
    Ok(())
}
