use crate::check::constrain::constraint::expected::{Expect, Expected};
use crate::check::constrain::constraint::iterator::Constraints;
use crate::check::result::TypeResult;

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
pub fn substitute(
    constraints: &mut Constraints,
    new: &Expected,
    old: &Expected,
    offset: usize,
    total: usize,
) -> TypeResult<()> {
    let mut constraint_pos = offset;
    trace!("{:width$} [subbing {}\\{}]  {}  <=  {}", "", offset, total, old, new, width = 30);

    for _ in 0..constraints.len() {
        let mut constr = constraints.pop_constr().expect("Cannot be empty");
        constraint_pos += 1;
        macro_rules! replace {
            ($left:expr, $new:expr) => {{
                let pos = format!("({}={}) ", constr.left.pos, constr.right.pos);
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

        let (sub_l, left) = recursive_substitute("l", &constr.left, old, new);
        let (sub_r, right) = recursive_substitute("r", &constr.right, old, new);

        constr.left = left;
        constr.right = right;
        if sub_l || sub_r {
            replace!(sub_l, constr)
        }

        constr.is_sub = constr.is_sub || sub_l || sub_r;
        constraints.push_constr(&constr);
    }

    Ok(())
}

fn recursive_substitute(
    side: &str,
    inspected: &Expected,
    old: &Expected,
    new: &Expected,
) -> (bool, Expected) {
    if inspected.expect.same_value(&old.expect) {
        return (true, new.clone());
    }

    match &inspected.expect {
        Expect::Access { entity, name } => {
            let (subs_e, entity) = recursive_substitute(side, entity, old, new);
            let (sub_n, name) = recursive_substitute(side, name, old, new);

            let expect = Expect::Access { entity: Box::from(entity), name: Box::from(name) };
            (subs_e || sub_n, Expected::new(inspected.pos, &expect))
        }
        Expect::Tuple { elements } => {
            let (any_substituted, elements) = substitute_vec(side, old, new, elements);
            (any_substituted, Expected::new(inspected.pos, &Expect::Tuple { elements }))
        }
        Expect::Collection { ty } => {
            let (subs_ty, ty) = recursive_substitute(side, ty, old, new);
            let expect = Expect::Collection { ty: Box::from(ty) };
            (subs_ty, Expected::new(inspected.pos, &expect))
        }
        Expect::Function { name, args } => {
            let (any_substituted, args) = substitute_vec(side, old, new, args);
            let func = Expect::Function { name: name.clone(), args };
            (any_substituted, Expected::new(inspected.pos, &func))
        }
        _ => (false, inspected.clone()),
    }
}

/// Substitute all in vector, and also returns True if any substituted.
fn substitute_vec(
    side: &str,
    old: &Expected,
    new: &Expected,
    elements: &[Expected],
) -> (bool, Vec<Expected>) {
    let elements = elements
        .iter()
        .map(|e| recursive_substitute(side, e, old, new));

    (elements.clone().any(|(i, _)| i), elements.map(|(_, i)| i).collect())
}
