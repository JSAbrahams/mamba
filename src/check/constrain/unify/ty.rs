use EitherOrBoth::Both;
use itertools::{EitherOrBoth, Itertools};

use crate::check::constrain::constraint::{Constraint, ConstrVariant};
use crate::check::constrain::constraint::expected::Expect::{Collection, Raises, Tuple, Type};
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::constraint::iterator::Constraints;
use crate::check::constrain::Unified;
use crate::check::constrain::unify::expression::substitute::substitute;
use crate::check::constrain::unify::link::unify_link;
use crate::check::context::{Context, LookupClass};
use crate::check::name::{Any, ColType, IsSuperSet, Name, Union};
use crate::check::name::name_variant::NameVariant;
use crate::check::result::{TypeErr, TypeResult};
use crate::common::position::Position;

pub fn unify_type(
    constraint: &Constraint,
    constraints: &mut Constraints,
    ctx: &Context,
    total: usize,
) -> Unified {
    let (left, right) = (&constraint.left, &constraint.right);
    let count = if constraints.len() <= total { total - constraints.len() } else { 0 };

    match (&left.expect, &right.expect) {
        (Type { name: l_ty }, Type { name: r_ty }) => {
            println!("{} = {}, {:?}", l_ty, r_ty, constraint.superset);
            let left_is_super = (constraint.superset == ConstrVariant::Left)
                && l_ty.is_superset_of(r_ty, ctx, left.pos)? || l_ty == &Name::any();
            let right_is_super = (constraint.superset == ConstrVariant::Right)
                && r_ty.is_superset_of(l_ty, ctx, left.pos)? || r_ty == &Name::any();

            if l_ty.is_temporary() {
                let mut constr =
                    substitute_ty(right.pos, r_ty, left.pos, l_ty, constraints, count, total)?;
                unify_link(&mut constr, ctx, total)
            } else if r_ty.is_temporary() {
                let mut constr =
                    substitute_ty(left.pos, l_ty, right.pos, r_ty, constraints, count, total)?;
                unify_link(&mut constr, ctx, total)
            } else if left_is_super || right_is_super {
                ctx.class(l_ty, left.pos)?;
                ctx.class(r_ty, right.pos)?;

                constraints.push_ty(left.pos, &l_ty.union(r_ty));
                constraints.push_ty(right.pos, &l_ty.union(r_ty));
                unify_link(constraints, ctx, total)
            } else if constraint.superset == ConstrVariant::Left {
                let msg = format!("Unifying two types: Expected {}, was {}", left, right);
                Err(vec![TypeErr::new(left.pos, &msg)])
            } else {
                let msg = format!("Unifying two types: Expected {}, was {}", right, left);
                Err(vec![TypeErr::new(right.pos, &msg)])
            }
        }

        (Raises { name: l_ty }, Raises { name: r_ty }) => {
            let left_confirmed_super = (constraint.superset == ConstrVariant::Left)
                && l_ty.is_superset_of(r_ty, ctx, left.pos)?;
            let right_confirmed_super = (constraint.superset == ConstrVariant::Right)
                && r_ty.is_superset_of(l_ty, ctx, left.pos)?;

            if l_ty.is_temporary() {
                let mut constr =
                    substitute_ty(right.pos, r_ty, left.pos, l_ty, constraints, count, total)?;
                unify_link(&mut constr, ctx, total)
            } else if r_ty.is_temporary() {
                let mut constr =
                    substitute_ty(left.pos, l_ty, right.pos, r_ty, constraints, count, total)?;
                unify_link(&mut constr, ctx, total)
            } else if left_confirmed_super || right_confirmed_super {
                ctx.class(l_ty, left.pos)?;
                ctx.class(r_ty, right.pos)?;
                unify_link(constraints, ctx, total)
            } else if constraint.superset == ConstrVariant::Left {
                let msg = format!("Unexpected raises '{}', may only be `{}`", l_ty, r_ty);
                Err(vec![TypeErr::new(left.pos, &msg)])
            } else {
                let msg = format!("Unexpected raises '{}', may only be `{}`", r_ty, l_ty);
                Err(vec![TypeErr::new(left.pos, &msg)])
            }
        }

        (Type { name }, Tuple { elements }) | (Tuple { elements }, Type { name }) => {
            for name_ty in &name.names {
                match &name_ty.variant {
                    NameVariant::Tuple(names) => {
                        if names.len() != elements.len() {
                            let msg = format!(
                                "Expected tuple with {} elements, was {}",
                                names.len(),
                                elements.len()
                            );
                            return Err(vec![TypeErr::new(left.pos, &msg)]);
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
                                        "Cannot assign {} elements to a tuple of size {}",
                                        elements.len(),
                                        names.len()
                                    );
                                    return Err(vec![TypeErr::new(left.pos, &msg)]);
                                }
                            }
                        }
                    }
                    _ => {
                        let msg =
                            format!("Unifying type and tuple: Expected {}, was {}", name, right);
                        return Err(vec![TypeErr::new(left.pos, &msg)]);
                    }
                }
            }

            unify_link(constraints, ctx, total)
        }

        (Collection { ty: l_ty }, Collection { ty: r_ty }) => {
            constraints.push("collection parameters", l_ty, r_ty);
            unify_link(constraints, ctx, total + 1)
        }
        (Tuple { elements: l_ty }, Tuple { elements: r_ty }) => {
            for pair in l_ty.iter().zip_longest(r_ty.iter()) {
                match &pair {
                    Both(name, exp) => constraints.push("tuple", name, exp),
                    _ => {
                        let msg = format!(
                            "Tuple sizes differ. Expected {} elements, was {}",
                            l_ty.len(),
                            r_ty.len()
                        );
                        return Err(vec![TypeErr::new(left.pos, &msg)]);
                    }
                }
            }
            unify_link(constraints, ctx, total + 1)
        }

        (l_exp, r_exp) => match (l_exp, r_exp) {
            (Collection { ty }, Type { name }) => {
                if let Some(col_ty) = name.col_type(ctx, right.pos)? {
                    let expect = Type { name: col_ty };
                    constraints.push("collection type", ty, &Expected::new(left.pos, &expect));
                    unify_link(constraints, ctx, total + 1)
                } else {
                    let msg = format!("Unifying type: Expected {}, was {}", left, right);
                    Err(vec![TypeErr::new(left.pos, &msg)])
                }
            }
            (Type { name }, Collection { ty }) => {
                if let Some(col_ty) = name.col_type(ctx, left.pos)? {
                    let expect = Type { name: col_ty };
                    constraints.push("collection type", &Expected::new(left.pos, &expect), ty);
                    unify_link(constraints, ctx, total + 1)
                } else {
                    let msg = format!("Unifying type: Expected {}, was {}", left, right);
                    Err(vec![TypeErr::new(left.pos, &msg)])
                }
            }

            // Ignore raises
            (Type { .. }, Raises { .. }) | (Raises { .. }, Type { .. }) => unify_link(constraints, ctx, total),

            _ => {
                if l_exp.is_none() && r_exp.is_none() {
                    unify_link(constraints, ctx, total)
                } else {
                    let msg = format!("Unifying type: Expected {}, was {}", left, right);
                    Err(vec![TypeErr::new(left.pos, &msg)])
                }
            }
        },
    }
}

fn substitute_ty(
    new_pos: Position,
    new: &Name,
    old_pos: Position,
    old: &Name,
    constraints: &mut Constraints,
    offset: usize,
    total: usize,
) -> TypeResult<Constraints> {
    let new = Expected::new(new_pos, &Type { name: new.clone() });
    let old = Expected::new(old_pos, &Type { name: old.clone() });
    substitute(&new, &old, constraints, offset, total)
}
