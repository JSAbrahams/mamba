use crate::check::constrain::constraint::expected::Expect::{Collection, ExpressionAny, Nullable,
                                                            Raises, Statement, Stringy, Truthy,
                                                            Type};
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::constraint::iterator::Constraints;
use crate::check::constrain::unify::unify_link::unify_link;
use crate::check::constrain::Unified;
use crate::check::context::{function, Context};
use crate::check::result::{TypeErr, TypeResult};
use crate::check::ty::name::TypeName;
use crate::common::delimit::comma_delimited;
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

        (Type { type_name }, Truthy) => {
            let expr_ty = ctx.lookup_class(type_name, &left.pos)?;
            expr_ty.function(&TypeName::from(function::TRUTHY), &left.pos)?;
            unify_link(constraints, ctx, total)
        }
        (Type { type_name }, Stringy) => {
            let expr_ty = ctx.lookup_class(type_name, &left.pos)?;
            expr_ty.function(&TypeName::from(function::STR), &left.pos)?;
            unify_link(constraints, ctx, total)
        }
        (Type { type_name: l_ty }, Type { type_name: r_ty }) => {
            if l_ty.is_superset(r_ty)
                || ctx.lookup_class(&r_ty, &right.pos)?.has_parent(&l_ty, ctx, &left.pos)?
            {
                ctx.lookup_class(l_ty, &left.pos)?;
                unify_link(constraints, ctx, total)
            } else {
                // TODO construct error based on type of constraint
                let msg = format!("Expected '{}', found '{}'", l_ty, r_ty);
                Err(vec![TypeErr::new(&left.pos, &msg)])
            }
        }

        (Type { type_name }, Raises { raises }) =>
            if raises.contains(type_name) {
                unify_link(constraints, ctx, total)
            } else {
                let msg = format!(
                    "Unexpected raises '{}', must be one of: {}",
                    type_name,
                    comma_delimited(raises)
                );
                Err(vec![TypeErr::new(&left.pos, &msg)])
            },

        (Type { type_name }, Nullable) =>
            if type_name.is_nullable() {
                unify_link(constraints, ctx, total)
            } else {
                let msg = format!("Expected '{}', found '{}'", type_name.as_nullable(), type_name);
                Err(vec![TypeErr::new(&left.pos, &msg)])
            },

        (Type { type_name }, Collection { ty }) => {
            let (mut constr, added) = check_iter(type_name, ty, ctx, constraints, &left.pos)?;
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
        (Statement, Statement) => unify_link(constraints, ctx, total),

        (l_exp, r_exp) => {
            let msg = format!("Expected '{}', found '{}'", l_exp, r_exp);
            Err(vec![TypeErr::new(&left.pos, &msg)])
        }
    }
}

fn check_iter(
    type_name: &TypeName,
    ty: &Expected,
    ctx: &Context,
    constr: &mut Constraints,
    pos: &Position
) -> TypeResult<(Constraints, usize)> {
    let f_name = TypeName::from(function::ITER);
    let mut added = 0;

    for fun in ctx.lookup_class(type_name, pos)?.function(&f_name, pos)? {
        let msg = format!("{} __iter__ type undefined", type_name);
        let f_ret_ty = fun.ty().ok_or_else(|| TypeErr::new(&pos, &msg))?;

        let f_name = TypeName::from(function::NEXT);
        for fun in ctx.lookup_class(&f_ret_ty, pos)?.function(&f_name, pos)? {
            let f_ret_ty = fun.ty().ok_or_else(|| TypeErr::new(&pos, &msg))?;
            added += 1;
            constr.eager_push(
                &Expected::new(&pos, &Type { type_name: type_name.clone() }),
                &Expected::new(&pos, &Type { type_name: f_ret_ty.clone() })
            );
        }
        added += 1;
        constr.eager_push(&ty, &Expected::new(&pos, &Type { type_name: f_ret_ty }));
    }

    Ok((constr.clone(), added))
}
