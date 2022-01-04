use EitherOrBoth::Both;
use itertools::{EitherOrBoth, Itertools};

use crate::check::constrain::constraint::expected::{Expect, Expected};
use crate::check::constrain::constraint::expected::Expect::{Collection, ExpressionAny, Raises, Tuple, Type};
use crate::check::constrain::constraint::iterator::Constraints;
use crate::check::constrain::Unified;
use crate::check::constrain::unify::link::unify_link;
use crate::check::context::{Context, LookupClass};
use crate::check::context::name::{IsSuperSet, NameVariant};
use crate::check::result::TypeErr;

pub fn unify_type(
    left: &Expected,
    right: &Expected,
    constraints: &mut Constraints,
    ctx: &Context,
    total: usize,
) -> Unified {
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

        (Type { name: l_ty }, Type { name: r_ty }) =>
            if l_ty.is_superset_of(r_ty, ctx, &left.pos)? {
                ctx.class(l_ty, &left.pos)?;
                unify_link(constraints, ctx, total)
            } else {
                let msg = format!("Expected a '{}', was a '{}'", l_ty, r_ty);
                Err(vec![TypeErr::new(&left.pos, &msg)])
            },

        (Type { name }, Raises { name: raises }) =>
            if raises.is_superset_of(name, ctx, &left.pos)? {
                unify_link(constraints, ctx, total)
            } else {
                let msg = format!("Unexpected raises '{}', may only be `{}`", name, raises);
                Err(vec![TypeErr::new(&left.pos, &msg)])
            },

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

        (l_exp, r_exp) => if l_exp.is_none() && r_exp.is_none() || l_exp.is_superset_of(r_exp, ctx)? {
            unify_link(constraints, ctx, total)
        } else {
            let msg = format!("Expected a '{}', was a '{}'", l_exp, r_exp);
            Err(vec![TypeErr::new(&left.pos, &msg)])
        }
    }
}
