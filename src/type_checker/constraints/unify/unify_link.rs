use std::collections::HashSet;
use std::ops::Deref;

use itertools::{EitherOrBoth, Itertools};

use crate::common::position::Position;
use crate::type_checker::constraints::constraint::expected::Expect::*;
use crate::type_checker::constraints::constraint::expected::Expected;
use crate::type_checker::constraints::constraint::Constraints;
use crate::type_checker::constraints::unify::substitute::substitute;
use crate::type_checker::constraints::Unified;
use crate::type_checker::context::function;
use crate::type_checker::context::function_arg::concrete::FunctionArg;
use crate::type_checker::context::Context;
use crate::type_checker::type_name::TypeName;
use crate::type_checker::type_result::TypeErr;

/// Unifies all constraints.

/// We use a mutable reference to constraints for performance reasons.
/// Otherwise, we have to make a entirely new copy of the list of all
/// constraints each time we do a recursive call to unify link.
pub fn unify_link(constr: &mut Constraints, sub: &Constraints, ctx: &Context) -> Unified {
    if let Some(constraint) = constr.constraints.pop() {
        match (&constraint.0.expect, &constraint.1.expect) {
            (ExpressionAny, ExpressionAny)
            | (Truthy, Truthy)
            | (RaisesAny, RaisesAny)
            | (Expression { .. }, ExpressionAny)
            | (ExpressionAny, Expression { .. })
            | (Truthy, ExpressionAny)
            | (ExpressionAny, Truthy)
            | (Type { .. }, ExpressionAny)
            | (ExpressionAny, Type { .. }) => unify_link(constr, sub, ctx),

            (Expression { .. }, Expression { .. })
            | (Type { .. }, Expression { .. })
            | (Expression { .. }, Type { .. }) => {
                let mut constr = substitute(&constraint.0, &constraint.1, &constr, false)?;
                let mut subst = Constraints::from(&constraint);
                subst.append(&substitute(&constraint.0, &constraint.1, &sub, true)?);
                unify_link(&mut constr, &subst, ctx)
            }

            (Type { type_name: left }, Type { type_name: right }) if left == right => {
                ctx.lookup(left, &constraint.0.pos)?;
                unify_link(constr, sub, ctx)
            }
            (Type { type_name }, Truthy) | (Truthy, Type { type_name }) => {
                let expr_ty = ctx.lookup(type_name, &constraint.0.pos)?;
                expr_ty.fun_args(&TypeName::from(function::python::TRUTHY), &constraint.0.pos)?;
                unify_link(constr, sub, ctx)
            }
            (Type { type_name: left }, Type { type_name: right }) => Err(vec![TypeErr::new(
                &Position::default(),
                &format!("Types not equal: {}, {}", left, right)
            )]),

            (Type { type_name }, Implements { type_name: f_name, args })
            | (Implements { type_name: f_name, args }, Type { type_name }) => {
                let expr_ty = ctx.lookup(type_name, &constraint.0.pos)?;
                let f_name = if f_name == type_name {
                    ctx.lookup(f_name, &constraint.0.pos)?;
                    TypeName::from(function::concrete::INIT)
                } else {
                    f_name.clone()
                };
                let possible = expr_ty.fun_args(&f_name, &constraint.0.pos)?;
                let mut constr = unify_fun_arg(possible, args, constr, &constraint.0.pos)?;
                unify_link(&mut constr, &sub, ctx)
            }

            (Truthy, Expression { .. }) | (Expression { .. }, Truthy) => unify_link(
                &mut substitute(&constraint.0, &constraint.1, constr, false)?,
                &substitute(&constraint.0, &constraint.1, &sub, true)?.add_constraint(&constraint),
                ctx
            ),

            (Mutable { expect }, _) => {
                constr.push(&Expected::new(&constraint.0.pos, expect.deref()), &constraint.0);
                unify_link(constr, sub, ctx)
            }
            (_, Mutable { expect }) => {
                constr.push(&constraint.0, &Expected::new(&constraint.1.pos, expect.deref()));
                unify_link(constr, sub, ctx)
            }

            _ => panic!(
                "Unexpected: {}={} : {} == {}",
                constraint.0.pos, constraint.1.pos, constraint.0.expect, constraint.1.expect
            )
        }
    } else {
        Ok(constr.clone())
    }
}

fn unify_fun_arg(
    possible: HashSet<Vec<FunctionArg>>,
    args: &Vec<Expected>,
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
                    return Err(vec![TypeErr::new(&pos, "Expected argument")]),
                EitherOrBoth::Right(expected) =>
                    return Err(vec![TypeErr::new(&expected.pos, "Unexpected argument")]),
                _ => {}
            }
        }
    }

    Ok(constr)
}
