use EitherOrBoth::Both;
use itertools::{EitherOrBoth, Itertools};

use crate::check::constrain::constraint::{Constraint, ConstrVariant};
use crate::check::constrain::constraint::expected::Expect::{Collection, Tuple, Type};
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::constraint::iterator::Constraints;
use crate::check::constrain::Unified;
use crate::check::constrain::unify::expression::substitute::substitute;
use crate::check::constrain::unify::finished::Finished;
use crate::check::constrain::unify::link::unify_link;
use crate::check::context::{Context, LookupClass};
use crate::check::name::{Any, ColType, IsSuperSet, Name, Union};
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
    let (left, right) = (&constraint.left, &constraint.right);
    let count = if constraints.len() <= total { total - constraints.len() } else { 0 };

    match (&left.expect, &right.expect) {
        (Type { name: l_ty }, Type { name: r_ty }) => {
            let left_is_super = (constraint.superset == ConstrVariant::Left)
                && l_ty.is_superset_of(r_ty, ctx, left.pos)? || l_ty == &Name::any();
            let right_is_super = (constraint.superset == ConstrVariant::Right)
                && r_ty.is_superset_of(l_ty, ctx, left.pos)? || r_ty == &Name::any();

            if l_ty.is_temporary() {
                substitute_ty(right.pos, r_ty, left.pos, l_ty, constraints, count, total)?;
                unify_link(constraints, finished, ctx, total)
            } else if r_ty.is_temporary() {
                substitute_ty(left.pos, l_ty, right.pos, r_ty, constraints, count, total)?;
                unify_link(constraints, finished, ctx, total)
            } else if left_is_super || right_is_super {
                ctx.class(l_ty, left.pos)?;
                ctx.class(r_ty, right.pos)?;

                finished.push_ty(left.pos, &l_ty.union(r_ty));
                finished.push_ty(right.pos, &l_ty.union(r_ty));
                unify_link(constraints, finished, ctx, total)
            } else if constraint.superset == ConstrVariant::Left {
                Err(unify_type_message(&constraint.msg, left, right))
            } else {
                Err(unify_type_message(&constraint.msg, right, left))
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

        (Collection { ty: l_ty }, Collection { ty: r_ty }) => {
            constraints.push("collection parameters", l_ty, r_ty);
            unify_link(constraints, finished, ctx, total + 1)
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

        (l_exp, r_exp) => match (l_exp, r_exp) {
            (Collection { ty }, Type { name }) => {
                if let Some(col_ty) = name.col_type(ctx, right.pos)? {
                    let expect = Type { name: col_ty };
                    constraints.push("collection type", ty, &Expected::new(left.pos, &expect));
                    unify_link(constraints, finished, ctx, total + 1)
                } else {
                    Err(unify_type_message(&constraint.msg, left, right))
                }
            }
            (Type { name }, Collection { ty }) => {
                if let Some(col_ty) = name.col_type(ctx, left.pos)? {
                    let expect = Type { name: col_ty };
                    constraints.push("collection type", &Expected::new(left.pos, &expect), ty);
                    unify_link(constraints, finished, ctx, total + 1)
                } else {
                    Err(unify_type_message(&constraint.msg, left, right))
                }
            }

            _ if l_exp.is_none() && r_exp.is_none() => unify_link(constraints, finished, ctx, total),
            _ => Err(unify_type_message(&constraint.msg, left, right))
        },
    }
}

pub fn unify_type_message(cause_msg: &str, sup: &Expected, child: &Expected) -> Vec<TypeErr> {
    let msg = format!("Expected {sup}, was {child}");
    vec![TypeErr::new(child.pos, &msg).with_cause(&cause_msg, sup.pos)]
}

fn substitute_ty(
    new_pos: Position,
    new: &Name,
    old_pos: Position,
    old: &Name,
    constraints: &mut Constraints,
    offset: usize,
    total: usize,
) -> TypeResult<()> {
    let new = Expected::new(new_pos, &Type { name: new.clone() });
    let old = Expected::new(old_pos, &Type { name: old.clone() });
    substitute(constraints, &new, &old, offset, total)
}
