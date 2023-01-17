use itertools::Itertools;

use crate::check::constrain::constraint::iterator::Constraints;
use crate::check::constrain::Unified;
use crate::check::constrain::unify::finished::Finished;
use crate::check::constrain::unify::link::unify_link;
use crate::check::context::Context;
use crate::common::delimit::newline_delimited;

pub mod finished;

mod link;

mod function;
mod ty;
mod expression;

pub fn unify(all_constraints: &[Constraints], ctx: &Context) -> Unified {
    let mut count = 1;
    let mut finished = Finished::default();
    let (_, errs): (Vec<_>, Vec<_>) = all_constraints
        .iter()
        .map(|constraints| {
            trace!("[unifying set {}\\{}: {} (branched at {})]", count, all_constraints.len(), constraints.msg, constraints.pos);
            count += 1;
            unify_link(&mut constraints.clone(), &mut finished, ctx, constraints.len()).map_err(|e| {
                trace!(
                    "[error unifying set {}\\{}:{}]",
                    count - 1,
                    all_constraints.len(),
                    newline_delimited(e.clone().into_iter().map(|e| {
                        let pos = e.pos.map_or_else(String::new, |pos| format!(" at {pos}: "));
                        format!("{pos}{}", e.msg)
                    }))
                );
                e
            })
        })
        .partition(Result::is_ok);

    if errs.is_empty() {
        Ok(finished)
    } else {
        let errs = errs.into_iter().flat_map(Result::unwrap_err);
        Err(errs.into_iter().unique().collect())
    }
}
