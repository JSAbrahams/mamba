use crate::check::constrain::constraint::expected::Expect::{Collection, ExpressionAny, Nullable,
                                                            Raises, Stringy, Truthy, Type};
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::constraint::iterator::Constraints;
use crate::check::constrain::unify::unify_link::unify_link;
use crate::check::constrain::Unified;
use crate::check::context::name::{AsNullable, DirectName, IsNullable, IsSuperSet, NameUnion};
use crate::check::context::{function, Context, LookupClass};
use crate::check::result::{TypeErr, TypeResult};
use crate::common::position::Position;

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
            let expr_ty = ctx.class(name, &left.pos)?;
            expr_ty.function(&DirectName::from(function::STR), &left.pos)?;
            unify_link(constraints, ctx, total)
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

        (Type { name }, Collection { ty: col_ty }) => {
            let (mut constr, added) = check_iter(name, col_ty, ctx, constraints, &left.pos)?;
            unify_link(&mut constr, ctx, total + added)
        }
        (Collection { ty: l_ty }, Collection { ty: r_ty }) => {
            constraints.eager_push(&l_ty, &r_ty);
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

fn check_iter(
    ty: &NameUnion,
    col_ty: &Expected,
    ctx: &Context,
    constr: &mut Constraints,
    pos: &Position
) -> TypeResult<(Constraints, usize)> {
    let f_name = DirectName::from(function::ITER);
    let mut added = 0;

    for fun in ctx.class(ty, pos)?.function(&f_name, pos)?.union {
        let f_name = DirectName::from(function::NEXT);
        for fun in ctx.class(&fun.ret_ty, pos)?.function(&f_name, pos)?.union {
            added += 1;
            constr.eager_push(
                &Expected::new(&pos, &Type { name: ty.clone() }),
                &Expected::new(&pos, &Type { name: fun.ret_ty })
            );
        }
        added += 1;
        constr.eager_push(&col_ty, &Expected::new(&pos, &Type { name: fun.ret_ty }));
    }

    Ok((constr.clone(), added))
}
