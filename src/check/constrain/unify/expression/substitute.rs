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
    identifiers: &[String],
    old: &Expected,
    new: &Expected,
    constraints: &mut Constraints
) -> TypeResult<Constraints> {
    // TODO deal with tuples of identifiers
    let mut substituted = Constraints::new(&constraints.in_class);
    let identifiers = Vec::from(identifiers);

    while let Some(mut constr) = constraints.pop_constr() {
        let old_constr = constr.clone();
        macro_rules! replace {
            ($new:expr) => {{
                let pos =
                    format!("({}={}) ", old_constr.parent.pos.start, old_constr.child.pos.start);
                trace!("{:width$} [substitute] {} ===> {}", pos, old_constr, $new, width = 17);
            }};
        };

        if !constr.ids.is_empty() && constr.ids == identifiers {
            let (sub_r, child) = recursive_substitute("r", &constr.child, old, new);

            constr.child = child;
            constr.is_sub = constr.is_sub || sub_r;
            if sub_r {
                replace!(constr)
            }

            substituted.push_constr(&constr);
            break;
        } else {
            let (sub_l, parent) = recursive_substitute("l", &constr.parent, old, new);
            let (sub_r, child) = recursive_substitute("r", &constr.child, old, new);

            constr.parent = parent;
            constr.child = child;
            constr.is_sub = constr.is_sub || sub_l || sub_r;
            if sub_l || sub_r {
                replace!(constr)
            }

            substituted.push_constr(&constr)
        }
    }

    substituted.append(constraints);
    Ok(substituted)
}

fn recursive_substitute(
    side: &str,
    inspected: &Expected,
    old: &Expected,
    new: &Expected
) -> (bool, Expected) {
    if is_expr_and_structurally_eq(&inspected.expect, &old.expect) {
        return (true, new.clone());
    }

    match &inspected.expect {
        Expect::Access { entity, name } => {
            let (subs_e, entity) = recursive_substitute(side, entity, old, new);
            let (sub_n, name) = recursive_substitute(side, name, old, new);

            let expect = Expect::Access { entity: Box::from(entity), name: Box::from(name) };
            (subs_e || sub_n, Expected::new(&inspected.pos, &expect))
        }
        Expect::Collection { size, ty } => {
            let (subs_ty, ty) = recursive_substitute(side, ty, old, new);
            let expect = Expect::Collection { size: size.clone(), ty: Box::from(ty) };
            (subs_ty, Expected::new(&inspected.pos, &expect))
        }
        Expect::Function { name, args } => {
            let mut any_substituted = false;
            let new_args = args.iter().map(|arg| {
                let (subs, arg) = recursive_substitute(side, arg, old, new);
                any_substituted = any_substituted || subs;
                arg
            });

            let func = Expect::Function { name: name.clone(), args: new_args.collect() };
            (any_substituted, Expected::new(&inspected.pos, &func))
        }
        _ => (false, inspected.clone())
    }
}

fn is_expr_and_structurally_eq(inspected: &Expect, old: &Expect) -> bool {
    match inspected {
        Expect::Expression { .. } => inspected.structurally_eq(&old),
        _ => false
    }
}
