use crate::check::constrain::constraint::expected::Expect::{Collection, ExpressionAny, Nullable,
                                                            Raises, Stringy, Truthy, Type};
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::constraint::iterator::Constraints;
use crate::check::constrain::unify::link::unify_link;
use crate::check::constrain::Unified;
use crate::check::context::name::{AsNullable, DirectName, IsNullable, IsSuperSet, NameVariant};
use crate::check::context::{function, Context, LookupClass};
use crate::check::result::TypeErr;

pub fn unify_type(
    left: &Expected,
    right: &Expected,
    constraints: &mut Constraints,
    ctx: &Context,
    total: usize
) -> Unified {
    match (&left.expect, &right.expect) {
        (_, ExpressionAny) => unify_link(constraints, ctx, total),
        (ExpressionAny, _) => unify_link(constraints, ctx, total),

        (Type { name }, Truthy) => {
            let class = ctx.class(name, &left.pos)?;
            class.function(&DirectName::from(function::TRUTHY), &left.pos)?;
            unify_link(constraints, ctx, total)
        }
        (Type { name }, Stringy) => {
            for name in name.names() {
                match &name.variant {
                    NameVariant::Single(name) => {
                        let class = ctx.class(name, &left.pos)?;
                        class.function(&DirectName::from(function::STR), &left.pos)?;
                    }
                    NameVariant::Tuple(names) =>
                        for name in names {
                            // Tuples are the exception, they can be printed
                            let class = ctx.class(name, &left.pos)?;
                            class.function(&DirectName::from(function::STR), &left.pos)?;
                        },
                    NameVariant::Fun(..) => {
                        let msg = format!("Cannot print '{}'", &left);
                        return Err(vec![TypeErr::new(&left.pos, &msg)]);
                    }
                }
            }

            unify_link(constraints, ctx, total)
        }
        (Collection { ty }, Stringy) => {
            constraints.push(ty, right);
            unify_link(constraints, ctx, total + 1)
        }

        (Type { name: l_ty }, Type { name: r_ty }) => {
            if l_ty.is_superset_of(r_ty, ctx, &left.pos)? {
                ctx.class(l_ty, &left.pos)?;
                unify_link(constraints, ctx, total)
            } else {
                // TODO construct error based on type of constraint
                let msg = format!("Expected '{}', found '{}'", l_ty, r_ty);
                Err(vec![TypeErr::new(&left.pos, &msg)])
            }
        }

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
            constraints.push(&l_ty, &r_ty);
            unify_link(constraints, ctx, total + 1)
        }

        (Truthy, Stringy) | (Stringy, Truthy) => unify_link(constraints, ctx, total),
        (Stringy, Nullable) | (Nullable, Stringy) => unify_link(constraints, ctx, total),
        (Stringy, Stringy) => unify_link(constraints, ctx, total),
        (Nullable, Nullable) => unify_link(constraints, ctx, total),

        (l_exp, r_exp) => {
            let msg = format!("Expected '{}', found '{}'", l_exp, r_exp);
            Err(vec![TypeErr::new(&left.pos, &msg)])
        }
    }
}
