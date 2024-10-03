use crate::check::constrain::constraint::expected::{Expect, Expected};
use crate::check::constrain::constraint::iterator::Constraints;
use crate::check::constrain::Unified;

pub type SubRes<T = Expected> = (bool, T);

/// Substitute old expression with new.
///
/// Only expression are ever substituted, everything else is left as is.
///
/// identifiers is used to signal when we should stop substituting.
/// Namely, if we encounter an identifier in a constraint, we abort
/// substitution and copy over all remaining constraints.
///
/// If identifier override detected, only substitute right hand side of
/// unification before ceasing substitution.
pub fn sub(
    constraints: &mut Constraints,
    new: &Expected,
    old: &Expected,
    offset: usize,
    total: usize,
) -> Unified<()> {
    let mut constraint_pos = offset;
    trace!(
        "{:width$} [subbing {}\\{}]  {}  <=  {}",
        "",
        offset,
        total,
        old,
        new,
        width = 30
    );

    for _ in 0..constraints.len() {
        let mut constr = constraints.pop_constr().expect("Cannot be empty");
        constraint_pos += 1;
        macro_rules! replace {
            ($left:expr, $new:expr) => {{
                let pos = format!("({}={}) ", constr.parent.pos, constr.child.pos);
                let side = if $left { "l" } else { "r" };
                trace!(
                    "{:width$} [subbed {}\\{} {}]  {}  =>  {}",
                    pos,
                    constraint_pos,
                    total,
                    side,
                    constr,
                    $new,
                    width = 32
                );
            }};
        }

        let (sub_l, left) = sub_recursive("l", &constr.parent, old, new);
        let (sub_r, right) = sub_recursive("r", &constr.child, old, new);

        constr.parent = left;
        constr.child = right;
        if sub_l || sub_r {
            replace!(sub_l, constr)
        }

        constr.is_sub = constr.is_sub || sub_l || sub_r;
        constraints.push_constr(&constr);
    }

    Ok(())
}

fn sub_recursive(side: &str, inspected: &Expected, old: &Expected, new: &Expected) -> SubRes {
    if inspected.expect.same_value(&old.expect) {
        return (true, new.clone());
    }

    match &inspected.expect {
        Expect::Access { entity, name } => {
            let (subs_e, entity) = sub_recursive(side, entity, old, new);
            let (sub_n, name) = sub_recursive(side, name, old, new);

            let expect = Expect::Access {
                entity: Box::from(entity),
                name: Box::from(name),
            };
            (subs_e || sub_n, Expected::new(inspected.pos, &expect))
        }
        Expect::Function { name, args } => {
            let (any_substituted, args) = sub_vec(side, old, new, args);
            let func = Expect::Function {
                name: name.clone(),
                args,
            };
            (any_substituted, Expected::new(inspected.pos, &func))
        }
        _ => (false, inspected.clone()),
    }
}

/// Substitute all in vector, and also returns True if any substituted.
fn sub_vec(
    side: &str,
    old: &Expected,
    new: &Expected,
    elements: &[Expected],
) -> SubRes<Vec<Expected>> {
    let elements = elements.iter().map(|e| sub_recursive(side, e, old, new));

    (
        elements.clone().any(|(i, _)| i),
        elements.map(|(_, i)| i).collect(),
    )
}
