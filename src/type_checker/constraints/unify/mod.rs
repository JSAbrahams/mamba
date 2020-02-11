use crate::common::delimit::newline_delimited;
use crate::type_checker::checker_result::TypeErr;
use crate::type_checker::constraints::constraint::iterator::Constraints;
use crate::type_checker::constraints::unify::unify_link::unify_link;
use crate::type_checker::constraints::Unified;
use crate::type_checker::context::Context;
use crate::type_checker::util::custom_delimited;
use itertools::Itertools;

mod substitute;
mod unify_link;

pub fn unify(all_constraints: &[Constraints], ctx: &Context) -> Unified<Vec<Constraints>> {
    let mut count = 1;
    let (oks, errs): (Vec<_>, Vec<_>) = all_constraints
        .iter()
        .map(|constraints| {
            println!(
                "[unifying set {}\\{}{}]",
                count,
                all_constraints.len(),
                if constraints.in_class.is_empty() {
                    String::new()
                } else {
                    format!("{}", custom_delimited(&constraints.in_class, " in ", " in "))
                }
            );
            count += 1;
            unify_link(&mut constraints.clone(), ctx, constraints.len()).map_err(|e| {
                println!(
                    "[error unifying set {}\\{}:{}]",
                    count - 1,
                    all_constraints.len(),
                    newline_delimited(e.clone().into_iter().map(|e| format!(
                        "{} {}",
                        if let Some(pos) = e.position {
                            format!(" at {}", pos)
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
        Ok(oks.into_iter().map(Result::unwrap).collect())
    } else {
        let errs: Vec<TypeErr> = errs.into_iter().flat_map(Result::unwrap_err).collect();
        Err(errs.into_iter().unique().collect())
    }
}
