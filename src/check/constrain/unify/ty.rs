use crate::check::constrain::constraint::expected::Expect::{Collection, ExpressionAny, Nullable,
                                                            Raises, Type};
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::constraint::iterator::Constraints;
use crate::check::constrain::unify::link::unify_link;
use crate::check::constrain::Unified;
use crate::check::context::name::{AsNullable, IsNullable, IsSuperSet};
use crate::check::context::{Context, LookupClass};
use crate::check::result::TypeErr;

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
                let msg = format!("Expected '{}', found '{}'", l_ty, r_ty);
                Err(vec![TypeErr::new(&left.pos, &msg)])
            },

        (Type { name }, Raises { name: raises }) =>
            if raises.is_superset_of(name, ctx, &left.pos)? {
                unify_link(constraints, ctx, total)
            } else {
                let msg = format!("Unexpected raises '{}', may only be `{}`", name, raises);
                Err(vec![TypeErr::new(&left.pos, &msg)])
            },

        (Type { name }, Nullable) =>
            if name.is_nullable() {
                unify_link(constraints, ctx, total)
            } else {
                let msg = format!("Expected '{}', found '{}'", name.as_nullable(), name);
                Err(vec![TypeErr::new(&left.pos, &msg)])
            },

        (Collection { ty: l_ty }, Collection { ty: r_ty }) => {
            constraints.push("collection parameters", &l_ty, &r_ty);
            unify_link(constraints, ctx, total + 1)
        }

        (Nullable, Nullable) => unify_link(constraints, ctx, total),

        (l_exp, r_exp) => {
            let msg = format!("Expected '{}', found '{}'", l_exp, r_exp);
            Err(vec![TypeErr::new(&left.pos, &msg)])
        }
    }
}
