use itertools::Itertools;
use log::trace;

use crate::check::constrain::constraint::iterator::ConstraintSet;
use crate::check::constrain::unify::finished::Finished;
use crate::check::constrain::unify::link::unify_link;
use crate::check::constrain::Unified;
use crate::check::context::Context;
use crate::check::result::TypeErr;
use crate::common::delimit::newline_delimited;

pub mod finished;

mod link;

mod expression;
mod function;
mod ty;

/// Unify each set in turn, collecting the types of every set into one [Finished].
/// A set is only copied out of the shared constraints when its turn comes, and dropped after.
pub fn unify(all_constraints: Vec<ConstraintSet>, ctx: &Context) -> Unified {
    let sets = all_constraints.len();
    let mut finished = Finished::default();
    let mut errs = vec![];

    for (count, set) in all_constraints.into_iter().enumerate() {
        trace!(
            "[unifying set {}\\{sets}: {} (branched at {})]",
            count + 1,
            set.msg,
            set.pos
        );
        let mut constraints = set.into_constraints();
        let total = constraints.len();
        if let Err(set_errs) = unify_link(&mut constraints, &mut finished, ctx, total) {
            trace_errs(count + 1, sets, &set_errs);
            errs.extend(set_errs);
        }
    }

    if errs.is_empty() {
        Ok(finished)
    } else {
        Err(errs.into_iter().unique().collect())
    }
}

fn trace_errs(count: usize, sets: usize, errs: &[TypeErr]) {
    trace!(
        "[error unifying set {count}\\{sets}:{}]",
        newline_delimited(errs.iter().map(|e| {
            let pos = e.pos.map_or_else(String::new, |pos| format!(" at {pos}: "));
            format!("{pos}{}", e.msg)
        }))
    )
}
