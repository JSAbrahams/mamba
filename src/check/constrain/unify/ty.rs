use crate::check::constrain::constraint::expected::Expect::{Collection, ExpressionAny, Nullable,
                                                            Raises, Tuple, Type};
use crate::check::constrain::constraint::expected::{Expect, Expected};
use crate::check::constrain::constraint::iterator::Constraints;
use crate::check::constrain::unify::link::unify_link;
use crate::check::constrain::Unified;
use crate::check::context::name::{AsNullable, IsNullable, IsSuperSet, NameVariant};
use crate::check::context::{Context, LookupClass};
use crate::check::result::TypeErr;
use itertools::{EitherOrBoth, Itertools};
use EitherOrBoth::Both;

pub fn unify_type(
    left: &Expected,
    right: &Expected,
    constraints: &mut Constraints,
    ctx: &Context,
    total: usize
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
                            let msg = format!(
                                "Cannot assign to tuple of size {} with tuple of size {}",
                                names.len(),
                                elements.len()
                            );
                            return Err(vec![TypeErr::new(&left.pos, &msg)]);
                        }

                        for pair in names.iter().zip_longest(elements.iter()) {
                            match &pair {
                                Both(name, exp) => {
                                    let l_ty = Expected::new(&left.pos, &Expect::Type {
                                        name: name.clone().clone()
                                    });
                                    constraints.push("tuple collection", &l_ty, &exp)
                                }
                                _ => {
                                    let msg = format!(
                                        "Cannot assign {} elements to a tuple of size {}",
                                        elements.len(),
                                        names.len()
                                    );
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

        (Type { name }, Nullable) =>
            if name.is_nullable() {
                unify_link(constraints, ctx, total)
            } else {
                let msg = format!("Expected a '{}', was a '{}'", name.as_nullable(), name);
                Err(vec![TypeErr::new(&left.pos, &msg)])
            },

        (Collection { ty: l_ty }, Collection { ty: r_ty }) => {
            constraints.push("collection parameters", &l_ty, &r_ty);
            unify_link(constraints, ctx, total + 1)
        }
        (Tuple { elements: l_ty }, Tuple { elements: r_ty }) => {
            for pair in l_ty.iter().zip_longest(r_ty.iter()) {
                match &pair {
                    Both(name, exp) => constraints.push("tuple collection", name, exp),
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

        (Nullable, Nullable) => unify_link(constraints, ctx, total),

        (l_exp, r_exp) => {
            let msg = format!("Expected a '{}', was a '{}'", l_exp, r_exp);
            Err(vec![TypeErr::new(&left.pos, &msg)])
        }
    }
}
