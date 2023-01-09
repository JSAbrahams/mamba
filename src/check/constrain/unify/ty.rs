use std::collections::HashMap;

use EitherOrBoth::Both;
use itertools::{EitherOrBoth, Itertools};

use crate::check::constrain::constraint::Constraint;
use crate::check::constrain::constraint::expected::{Expect, Expected};
use crate::check::constrain::constraint::expected::Expect::{Tuple, Type};
use crate::check::constrain::constraint::iterator::Constraints;
use crate::check::constrain::Unified;
use crate::check::constrain::unify::finished::Finished;
use crate::check::constrain::unify::link::unify_link;
use crate::check::context::{Context, LookupClass};
use crate::check::name::{Any, ContainsTemp, IsSuperSet, Name, Substitute};
use crate::check::name::name_variant::NameVariant;
use crate::check::result::{TypeErr, TypeResult};
use crate::common::position::Position;
use crate::common::result::WithCause;

pub fn unify_type(
    constraint: &Constraint,
    constraints: &mut Constraints,
    finished: &mut Finished,
    ctx: &Context,
    total: usize,
) -> Unified {
    let (left, right) = (&constraint.parent, &constraint.child);
    let count = if constraints.len() <= total { total - constraints.len() } else { 0 };

    match (&left.expect, &right.expect) {
        (Type { name: l_ty }, Type { name: r_ty }) => {
            if l_ty.is_temporary() {
                substitute_ty(right.pos, r_ty, left.pos, l_ty, constraints, count, total)?;
                return unify_link(constraints, finished, ctx, total);
            } else if r_ty.is_temporary() {
                substitute_ty(left.pos, l_ty, right.pos, r_ty, constraints, count, total)?;
                return unify_link(constraints, finished, ctx, total);
            } else if l_ty.contains_temp() {
                for (old, new) in l_ty.temp_map(r_ty, left.pos)? {
                    substitute_ty(left.pos, &new, right.pos, &old, constraints, count, total)?;
                }
                return unify_link(constraints, finished, ctx, total);
            } else if r_ty.contains_temp() {
                for (old, new) in r_ty.temp_map(l_ty, left.pos)? {
                    substitute_ty(left.pos, &new, right.pos, &old, constraints, count, total)?;
                }
                return unify_link(constraints, finished, ctx, total);
            }

            if l_ty.is_superset_of(r_ty, ctx, left.pos)? || l_ty == &Name::any() || r_ty == &Name::any() {
                ctx.class(l_ty, left.pos)?;
                ctx.class(r_ty, right.pos)?;

                finished.push_ty(ctx, left.pos, l_ty)?;
                finished.push_ty(ctx, right.pos, r_ty)?;
                unify_link(constraints, finished, ctx, total)
            } else {
                Err(unify_type_message(&constraint.msg, left, right))
            }
        }

        (Type { name }, Tuple { elements }) | (Tuple { elements }, Type { name }) => {
            for name_ty in &name.names {
                match &name_ty.variant {
                    NameVariant::Tuple(names) => {
                        if names.len() != elements.len() {
                            let msg = format!(
                                "In {}, expected tuple with {} elements, was {}",
                                constraint.msg,
                                names.len(),
                                elements.len()
                            );
                            return Err(unify_type_message(&msg, left, right));
                        }

                        for pair in names.iter().cloned().zip_longest(elements.iter()) {
                            match &pair {
                                Both(name, exp) => {
                                    let expect = Type { name: name.clone() };
                                    let l_ty = Expected::new(left.pos, &expect);
                                    constraints.push("tuple", &l_ty, exp)
                                }
                                _ => {
                                    let msg = format!(
                                        "In {}, Cannot assign {} elements to a tuple of size {}",
                                        constraint.msg,
                                        elements.len(),
                                        names.len()
                                    );
                                    return Err(unify_type_message(&msg, left, right));
                                }
                            }
                        }
                    }
                    _ => {
                        let msg = format!("Unifying type and tuple: Expected {name}, was {right}");
                        return Err(unify_type_message(&msg, left, right));
                    }
                }
            }

            unify_link(constraints, finished, ctx, total)
        }

        (Tuple { elements: l_ty }, Tuple { elements: r_ty }) => {
            for pair in l_ty.iter().zip_longest(r_ty.iter()) {
                match &pair {
                    Both(name, exp) => constraints.push("tuple", name, exp),
                    _ => {
                        let msg = format!(
                            "In {}, Tuple sizes differ. Expected {} elements, was {}",
                            constraint.msg,
                            l_ty.len(),
                            r_ty.len()
                        );
                        return Err(unify_type_message(&msg, left, right));
                    }
                }
            }
            unify_link(constraints, finished, ctx, total + 1)
        }

        _ => Err(unify_type_message(&constraint.msg, left, right))
    }
}

pub fn unify_type_message(cause_msg: &str, sup: &Expected, child: &Expected) -> Vec<TypeErr> {
    let msg = format!("Expected {sup}, was {child}");
    vec![TypeErr::new(child.pos, &msg).with_cause(cause_msg, sup.pos)]
}

fn substitute_ty(
    new_pos: Position,
    new: &Name,
    old_pos: Position,
    old: &Name,
    constraints: &mut Constraints,
    offset: usize,
    total: usize,
) -> Unified<()> {
    let mut constraint_pos = offset;
    let old_to_new = HashMap::from([(old.clone(), new.clone())]);
    trace!("{:width$} [subbing {}\\{}]  {}  <=  {}", "", offset, total, old, new, width = 30);

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

        let (sub_l, parent) = recursive_substitute_ty("l", &constr.parent, &old_to_new, new_pos)?;
        let (sub_r, child) = recursive_substitute_ty("r", &constr.child, &old_to_new, old_pos)?;

        constr.parent = parent;
        constr.child = child;
        if sub_l || sub_r {
            replace!(sub_l, constr)
        }

        constr.is_sub = constr.is_sub || sub_l || sub_r;
        constraints.push_constr(&constr);
    }

    Ok(())
}

fn recursive_substitute_ty(
    side: &str,
    inspected: &Expected,
    old_to_new: &HashMap<Name, Name>,
    pos: Position,
) -> TypeResult<(bool, Expected)> {
    Ok(match &inspected.expect {
        Expect::Access { entity, name } => {
            let (subs_e, entity) = recursive_substitute_ty(side, entity, old_to_new, pos)?;
            let (sub_n, name) = recursive_substitute_ty(side, name, old_to_new, pos)?;

            let expect = Expect::Access { entity: Box::from(entity), name: Box::from(name) };
            (subs_e || sub_n, Expected::new(inspected.pos, &expect))
        }
        Tuple { elements } => {
            let (any_substituted, elements) = substitute_vec_ty(side, old_to_new, elements, pos)?;
            (any_substituted, Expected::new(inspected.pos, &Tuple { elements }))
        }
        Expect::Function { name, args } => {
            let (any_substituted, args) = substitute_vec_ty(side, old_to_new, args, pos)?;
            let func = Expect::Function { name: name.clone(), args };
            (any_substituted, Expected::new(inspected.pos, &func))
        }
        Type { name } => {
            let new_name = name.substitute(old_to_new, pos)?;
            (*name != new_name, Expected::new(inspected.pos, &Type { name: new_name }))
        }
        _ => (false, inspected.clone()),
    })
}

fn substitute_vec_ty(
    side: &str,
    old_to_new: &HashMap<Name, Name>,
    elements: &[Expected],
    pos: Position,
) -> TypeResult<(bool, Vec<Expected>)> {
    let elements: Vec<(bool, Expected)> = elements
        .iter()
        .map(|e| recursive_substitute_ty(side, e, old_to_new, pos))
        .collect::<TypeResult<_>>()?;

    Ok((elements.iter().clone().any(|(i, _)| *i), elements.into_iter().map(|(_, i)| i).collect()))
}
