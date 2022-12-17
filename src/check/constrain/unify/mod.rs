use itertools::Itertools;

use crate::check::constrain::constraint::iterator::Constraints;
use crate::check::constrain::Unified;
use crate::check::constrain::unify::finished::Finished;
use crate::check::constrain::unify::link::unify_link;
use crate::check::context::Context;
use crate::common::delimit::{custom_delimited, newline_delimited};

pub mod finished;

mod link;

mod expression;
mod function;
mod ty;

pub fn unify(all_constraints: &[Constraints], ctx: &Context) -> Unified<Finished> {
    let mut count = 1;
    let mut finished = Finished::new();
    let (_, errs): (Vec<_>, Vec<_>) = all_constraints
        .iter()
        .map(|constraints| {
            trace!(
                "[unifying set {}\\{}{}]",
                count,
                all_constraints.len(),
                if constraints.in_class.is_empty() {
                    String::new()
                } else {
                    custom_delimited(&constraints.in_class, " in ", " in ")
                }
            );
            count += 1;
            unify_link(&mut constraints.clone(), &mut finished, ctx, constraints.len()).map_err(|e| {
                trace!(
                    "[error unifying set {}\\{}:{}]",
                    count - 1,
                    all_constraints.len(),
                    newline_delimited(e.clone().into_iter().map(|e| format!(
                        "{}{}",
                        if let Some(pos) = e.position {
                            format!(" at {}: ", pos)
                        } else {
                            String::new()
                        },
                        e.msg
                    )))
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
