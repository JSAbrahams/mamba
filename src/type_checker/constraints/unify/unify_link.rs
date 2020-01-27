use std::collections::HashSet;
use std::ops::Deref;

use itertools::{EitherOrBoth, Itertools};

use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::constraint::expected::Expect::*;
use crate::type_checker::constraints::constraint::expected::Expected;
use crate::type_checker::constraints::constraint::iterator::Constraints;
use crate::type_checker::constraints::unify::substitute::substitute;
use crate::type_checker::constraints::Unified;
use crate::type_checker::context::function;
use crate::type_checker::context::function_arg::concrete::FunctionArg;
use crate::type_checker::context::ty;
use crate::type_checker::context::Context;
use crate::type_checker::type_name::TypeName;
use crate::type_checker::type_result::TypeErr;

/// Unifies all constraints.

/// We use a mutable reference to constraints for performance reasons.
/// Otherwise, we have to make a entirely new copy of the list of all
/// constraints each time we do a recursive call to unify link.
pub fn unify_link(
    constr: &mut Constraints,
    sub: &Constraints,
    ctx: &Context,
    total: usize
) -> Unified {
    if let Some(constraint) = &constr.pop_constr() {
        let (left, right) = (constraint.parent.clone(), constraint.child.clone());
        println!(
            "{:width$} [solving {}\\{}{}] {} = {}",
            format!("({}={})", left.pos.start, right.pos.start),
            total - constr.len(),
            total,
            if constraint.flagged { " (flagged)" } else { "" },
            left.expect,
            right.expect,
            width = 13
        );

        match (&left.expect, &right.expect) {
            // trivially equal
            (l_expect, r_expect) if l_expect == r_expect => {
                let mut constr = substitute(&left, &right, constr)?;
                unify_link(&mut constr, sub, ctx, total)
            }
            (Expression { .. }, ExpressionAny) | (ExpressionAny, Expression { .. }) =>
                unify_link(constr, sub, ctx, total),
            (Type { .. }, ExpressionAny) | (ExpressionAny, Type { .. }) =>
                unify_link(constr, sub, ctx, total),
            (Truthy, ExpressionAny) | (ExpressionAny, Truthy) =>
                unify_link(constr, sub, ctx, total),

            (Expression { ast: AST { node: Node::Undefined, .. } }, Type { type_name })
                if !type_name.is_nullable() =>
                Err(vec![TypeErr::new(
                    &left.pos,
                    &format!("Expected {} but was {}", type_name.as_nullable(), type_name)
                )]),
            (Type { type_name }, Expression { ast: AST { node: Node::Undefined, .. } })
                if !type_name.is_nullable() =>
                Err(vec![TypeErr::new(
                    &right.pos,
                    &format!("Expected {} but was {}", type_name.as_nullable(), type_name)
                )]),

            (Expression { .. }, Expression { .. })
            | (Expression { .. }, Type { .. })
            | (Expression { .. }, Truthy) => {
                let mut constr = substitute(&left, &right, &constr)?;
                let mut subst = Constraints::default();
                subst.push(&left, &right);
                subst.append(&substitute(&left, &right, &sub)?);
                unify_link(&mut constr, &subst, ctx, total)
            }

            (Type { .. }, Expression { .. }) | (Truthy, Expression { .. }) => {
                let mut constr = substitute(&left, &right, &constr)?;
                let mut subst = Constraints::default();
                subst.push(&left, &right);
                subst.append(&substitute(&left, &right, &sub)?);
                unify_link(&mut constr, &subst, ctx, total)
            }

            (Type { type_name }, Truthy) | (Truthy, Type { type_name }) => {
                let expr_ty = ctx.lookup(type_name, &left.pos)?;
                expr_ty.fun_args(&TypeName::from(function::python::TRUTHY), &left.pos)?;
                unify_link(constr, sub, ctx, total)
            }
            (Type { type_name: l_ty }, Type { type_name: r_ty }) =>
                if l_ty.is_superset(r_ty) {
                    ctx.lookup(l_ty, &left.pos)?;
                    unify_link(constr, sub, ctx, total)
                } else {
                    // TODO construct error based on type of constraint
                    let msg = format!("Types not equal: {} != {}", l_ty, r_ty);
                    Err(vec![TypeErr::new(&left.pos, &msg)])
                },

            (Type { type_name }, Implements { type_name: f_name, args })
            | (Implements { type_name: f_name, args }, Type { type_name }) => {
                let expr_ty = ctx.lookup(type_name, &left.pos)?;
                let f_name = if f_name == type_name {
                    ctx.lookup(f_name, &left.pos)?;
                    TypeName::from(function::concrete::INIT)
                } else {
                    f_name.clone()
                };

                let possible = expr_ty.fun_args(&f_name, &left.pos)?;
                let mut constr = unify_fun_arg(possible, &args, constr, &right.pos)?;
                unify_link(&mut constr, &sub, ctx, total)
            }

            (Mutable { expect }, _) => {
                constr.push(&Expected::new(&left.pos, expect.deref()), &right);
                unify_link(constr, sub, ctx, total)
            }
            (_, Mutable { expect }) => {
                constr.push(&left, &Expected::new(&right.pos, expect.deref()));
                unify_link(constr, sub, ctx, total)
            }

            (Nullable { expect }, r_expect) => {
                let type_name = TypeName::from(ty::concrete::NONE);
                let nullable = Expected::new(&right.pos, &Type { type_name });

                constr.push(&Expected::new(&right.pos, r_expect.deref()), &nullable);
                constr.push(&Expected::new(&left.pos, expect.deref()), &right);
                unify_link(constr, sub, ctx, total)
            }
            (l_expect, Nullable { expect }) => {
                let type_name = TypeName::from(ty::concrete::NONE);
                let nullable = Expected::new(&left.pos, &Type { type_name });

                constr.push(&Expected::new(&left.pos, l_expect.deref()), &nullable);
                constr.push(&left, &Expected::new(&right.pos, expect.deref()));
                unify_link(constr, sub, ctx, total)
            }

            (Type { type_name }, Function { name, args })
            | (Function { name, args }, Type { type_name }) => {
                let expr_ty = ctx.lookup(type_name, &left.pos)?;
                let functions = expr_ty.anon_fun_params(&left.pos)?;

                for (f_args, f_ret_ty) in &functions {
                    for possible in f_args.iter().zip_longest(args.iter()) {
                        match possible {
                            EitherOrBoth::Both(type_name, expected) => {
                                let ty = Type { type_name: type_name.clone() };
                                let right = Expected::new(&left.pos, &ty);
                                constr.push(expected, &right);
                            }
                            EitherOrBoth::Left(_) | EitherOrBoth::Right(_) => {
                                let msg = format!(
                                    "{} arguments given to function which takes {} arguments",
                                    args.len(),
                                    f_args.len()
                                );
                                return Err(vec![TypeErr::new(&left.pos, &msg)]);
                            }
                        }
                    }
                }

                unify_link(constr, sub, ctx, total)
            }

            (Type { type_name }, HasField { name }) => {
                let expr_ty = ctx.lookup(type_name, &left.pos)?;
                let field_ty = expr_ty.field(name, &right.pos)?.ty()?;
                unify_link(constr, sub, ctx, total)
            }
            (HasField { name }, Type { type_name }) => {
                let expr_ty = ctx.lookup(type_name, &right.pos)?;
                let field_ty = expr_ty.field(name, &left.pos)?.ty()?;
                unify_link(constr, sub, ctx, total)
            }

            _ => {
                let pos = format!("({}={})", left.pos.start, right.pos.start);
                let count = format!("[reinserting {}\\{}]", total - constr.len(), total);
                println!("{:width$} {} {} = {}", pos, count, left.expect, right.expect, width = 15);

                // Defer to later point
                constr.reinsert(&constraint)?;
                unify_link(constr, sub, ctx, total)
            }
        }
    } else {
        Ok(constr.clone())
    }
}

fn unify_fun_arg(
    possible: HashSet<Vec<FunctionArg>>,
    args: &[Expected],
    constr: &Constraints,
    pos: &Position
) -> Unified {
    let mut constr = constr.clone();

    for f_args in possible {
        for either_or_both in f_args.iter().zip_longest(args.iter()) {
            match either_or_both {
                EitherOrBoth::Both(fun_arg, expected) => {
                    let ty = &fun_arg.ty.as_ref();
                    let type_name = ty
                        .ok_or_else(|| {
                            TypeErr::new(
                                &expected.pos,
                                "Function argument must have type parameters"
                            )
                        })?
                        .clone();
                    constr.push(&expected, &Expected::new(&expected.pos, &Type { type_name }))
                }
                EitherOrBoth::Left(fun_arg) if !fun_arg.has_default =>
                    return Err(vec![TypeErr::new(
                        &pos,
                        &format!("Expected argument: expected {}", fun_arg)
                    )]),
                EitherOrBoth::Right(expected) =>
                    return Err(vec![TypeErr::new(
                        &pos,
                        &format!(
                            "Unexpected argument, function takes only {} arguments",
                            f_args.len()
                        )
                    )]),
                _ => {}
            }
        }
    }

    Ok(constr)
}
