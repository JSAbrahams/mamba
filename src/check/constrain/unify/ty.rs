use EitherOrBoth::Both;
use itertools::{EitherOrBoth, Itertools};

use crate::check::constrain::constraint::{Constraint, ConstrVariant};
use crate::check::constrain::constraint::expected::{Expect, Expected};
use crate::check::constrain::constraint::expected::Expect::{Collection, ExpressionAny, Raises, Tuple, Type};
use crate::check::constrain::constraint::iterator::Constraints;
use crate::check::constrain::Unified;
use crate::check::constrain::unify::link::unify_link;
use crate::check::context::{Context, LookupClass};
use crate::check::context::name::{IsSuperSet, NameVariant};
use crate::check::result::TypeErr;

pub fn unify_type(constraint: &Constraint, constraints: &mut Constraints, ctx: &Context, total: usize) -> Unified {
    let (left, right) = (&constraint.left, &constraint.right);
    match (&left.expect, &right.expect) {
        (ExpressionAny, ty) | (ty, ExpressionAny) => match ty {
            Type { name } =>
                if name.is_empty() {
                    let msg = format!("Expected an expression, but was '{}'", name);
                    Err(vec![TypeErr::new(&left.pos, &msg)])
                } else {
                    unify_link(constraints, ctx, total)
                },
            _ => unify_link(constraints, ctx, total)
        },

        (Type { name: l_ty }, Type { name: r_ty }) => {
            let left_is_super = (constraint.superset == ConstrVariant::Left)
                && l_ty.is_superset_of(r_ty, ctx, &left.pos)?;
            let right_is_super = (constraint.superset == ConstrVariant::Right)
                && r_ty.is_superset_of(l_ty, ctx, &left.pos)?;
            let either_is_super = (constraint.superset == ConstrVariant::Either)
                && (l_ty.is_superset_of(r_ty, ctx, &left.pos)?
                || r_ty.is_superset_of(l_ty, ctx, &left.pos)?);

            if left_is_super || right_is_super || either_is_super {
                ctx.class(l_ty, &left.pos)?;
                ctx.class(r_ty, &right.pos)?;
                unify_link(constraints, ctx, total)
            } else if left_is_super {
                let msg = format!("Expected a '{}', was a '{}'", l_ty, r_ty);
                Err(vec![TypeErr::new(&left.pos, &msg)])
            } else {
                let msg = format!("Expected a '{}', was a '{}'", r_ty, l_ty);
                Err(vec![TypeErr::new(&left.pos, &msg)])
            }
        }

        (Raises { name: l_ty }, Raises { name: r_ty }) => {
            let left_confirmed_super = (constraint.superset == ConstrVariant::Left
                || constraint.superset == ConstrVariant::Either)
                && l_ty.is_superset_of(r_ty, ctx, &left.pos)?;
            let right_confirmed_super = (constraint.superset == ConstrVariant::Right)
                && r_ty.is_superset_of(l_ty, ctx, &left.pos)?;

            if left_confirmed_super || right_confirmed_super {
                ctx.class(l_ty, &left.pos)?;
                ctx.class(r_ty, &right.pos)?;
                unify_link(constraints, ctx, total)
            } else if left_confirmed_super {
                let msg = format!("Unexpected raises '{}', may only be `{}`", l_ty, r_ty);
                Err(vec![TypeErr::new(&left.pos, &msg)])
            } else {
                let msg = format!("Unexpected raises '{}', may only be `{}`", r_ty, l_ty);
                Err(vec![TypeErr::new(&left.pos, &msg)])
            }
        }

        (Type { name }, Tuple { elements }) | (Tuple { elements }, Type { name }) => {
            for name_ty in name.names() {
                match name_ty.variant {
                    NameVariant::Tuple(names) => {
                        if names.len() != elements.len() {
                            let msg = format!("Expected tuple with {} elements, was {}", names.len(), elements.len());
                            return Err(vec![TypeErr::new(&left.pos, &msg)]);
                        }

                        for pair in names.iter().zip_longest(elements.iter()) {
                            match &pair {
                                Both(name, exp) => {
                                    let expect = Expect::Type { name: name.clone().clone() };
                                    let l_ty = Expected::new(&left.pos, &expect);
                                    constraints.push("tuple", &l_ty, &exp)
                                }
                                _ => {
                                    let msg = format!("Cannot assign {} elements to a tuple of size {}", elements.len(), names.len());
                                    return Err(vec![TypeErr::new(&left.pos, &msg)]);
                                }
                            }
                        }
                    }
                    _ => {
                        let msg = format!("Expected a '{}', was a '{}'", name, right);
                        return Err(vec![TypeErr::new(&left.pos, &msg)]);
                    }
                }
            }
            unify_link(constraints, ctx, total)
        }

        (Collection { ty: l_ty }, Collection { ty: r_ty }) => {
            constraints.push("collection parameters", &l_ty, &r_ty);
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
                        return Err(vec![TypeErr::new(&left.pos, &msg)]);
                    }
                }
            }
            unify_link(constraints, ctx, total + 1)
        }

        (l_exp, r_exp) => if l_exp.is_none() && r_exp.is_none() {
            unify_link(constraints, ctx, total)
        } else {
            let msg = format!("Expected a '{}', was a '{}'", l_exp, r_exp);
            Err(vec![TypeErr::new(&left.pos, &msg)])
        }
    }
}
