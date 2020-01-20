use std::collections::HashSet;
use std::ops::Deref;

use itertools::{EitherOrBoth, Itertools};

use crate::common::position::Position;
use crate::type_checker::constraints::cons::Expect::*;
use crate::type_checker::constraints::cons::{Constraint, Constraints, Expected};
use crate::type_checker::constraints::Unified;
use crate::type_checker::context::function;
use crate::type_checker::context::function_arg::concrete::FunctionArg;
use crate::type_checker::context::Context;
use crate::type_checker::type_name::TypeName;
use crate::type_checker::type_result::{TypeErr, TypeResult};

/// Unifies all constraints.

/// We use a mutable reference to constraints for performance reasons.
/// Otherwise, we have to make a entirely new copy of the list of all
/// constraints each time we do a recursive call to unify link.
pub fn unify_link(constr: &mut Constraints, sub: &Constraints, ctx: &Context) -> Unified {
    while let Some(constraint) = constr.constraints.pop() {
        let constraint = match (&constraint.0.expect, &constraint.1.expect) {
            (ExpressionAny, ExpressionAny) | (Truthy, Truthy) | (RaisesAny, RaisesAny) =>
                Ok(Constraints::from(&constraint)),

            (Expression { .. }, ExpressionAny) | (ExpressionAny, Expression { .. }) =>
                Ok(Constraints::from(&constraint)),
            (Truthy, ExpressionAny) | (ExpressionAny, Truthy) => Ok(Constraints::from(&constraint)),
            (Type { .. }, ExpressionAny) | (ExpressionAny, Type { .. }) =>
                Ok(Constraints::from(&constraint)),

            (Expression { .. }, Expression { .. })
            | (Type { .. }, Expression { .. })
            | (Expression { .. }, Type { .. }) => {
                let mut constr = substitute(&constraint.0, &constraint.1, &constr)?;
                let mut subst = Constraints::from(&constraint);
                subst.append(&substitute(&constraint.0, &constraint.1, &sub)?);
                unify_link(&mut constr, &subst, ctx)
            }

            (Type { type_name: left }, Type { type_name: right }) if left == right => {
                ctx.lookup(left, &constraint.0.pos)?;
                Ok(Constraints::from(&constraint))
            }
            (Type { type_name }, Truthy) | (Truthy, Type { type_name }) => {
                let expr_ty = ctx.lookup(type_name, &constraint.0.pos)?;
                expr_ty.fun_args(&TypeName::from(function::python::TRUTHY), &constraint.0.pos)?;
                Ok(Constraints::from(&constraint))
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
                let possible: HashSet<Vec<FunctionArg>> =
                    expr_ty.fun_args(&f_name, &constraint.0.pos)?;

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
                                constr.push(
                                    &expected,
                                    &Expected::new(&expected.pos, &Type { type_name })
                                )
                            }
                            EitherOrBoth::Left(fun_arg) if !fun_arg.has_default =>
                                return Err(vec![TypeErr::new(
                                    &constraint.0.pos,
                                    "Expected argument"
                                )]),
                            EitherOrBoth::Right(expected) =>
                                return Err(vec![TypeErr::new(
                                    &expected.pos,
                                    "Unexpected argument"
                                )]),
                            _ => {}
                        }
                    }
                }

                unify_link(constr, &sub, ctx)
            }

            (Truthy, Expression { .. }) | (Expression { .. }, Truthy) => unify_link(
                &mut substitute(&constraint.0, &constraint.1, constr)?,
                &substitute(&constraint.0, &constraint.1, &sub)?.add_constraint(&constraint),
                ctx
            ),

            (Mutable { expect }, _) => {
                let constraint =
                    Constraint(Expected::new(&constraint.0.pos, expect.deref()), constraint.1);
                Ok(Constraints::from(&constraint))
            }
            (_, Mutable { expect }) => {
                let constraint =
                    Constraint(constraint.0, Expected::new(&constraint.1.pos, expect.deref()));
                Ok(Constraints::from(&constraint))
            }

            _ => panic!(
                "Unexpected: {}={} : {:?} == {:?}",
                constraint.0.pos, constraint.1.pos, constraint.0.expect, constraint.1.expect
            )
        }?;

        for constraint in &constraint.constraints {
            println!(
                "{:width$} {:?} == {:?}",
                format!("({},{})", constraint.0.pos, constraint.1.pos),
                constraint.0.expect,
                constraint.1.expect,
                width = 35
            );
        }
    }

    Ok(constr.clone())
}

fn substitute(old: &Expected, new: &Expected, constr: &Constraints) -> TypeResult<Constraints> {
    println!(
        "{:width$} subst {:?} with {:?}",
        format!("({}<={})", old.pos, new.pos),
        old.expect,
        new.expect,
        width = 29
    );
    sub_inner(old, new, &mut constr.clone())
}

fn sub_inner(old: &Expected, new: &Expected, constr: &mut Constraints) -> TypeResult<Constraints> {
    if let Some(constraint) = constr.constraints.pop() {
        let (left, right) = (constraint.0, constraint.1);
        let left = if &left == old {
            println!(
                "{:width$} replacing {:?} with {:?}",
                format!("({}<={})", old.pos, new.pos),
                old.expect,
                new.expect,
                width = 29
            );
            Expected::new(&left.pos, &new.expect)
        } else {
            left
        };
        let right = if &right == old {
            println!(
                "{:width$} replacing {:?} with {:?}",
                format!("({}<={})", old.pos, new.pos),
                old.expect,
                new.expect,
                width = 29
            );
            Expected::new(&right.pos, &new.expect)
        } else {
            right
        };
        let mut unified = Constraints::new().add(&left, &right);
        Ok(unified.append(&sub_inner(old, new, constr)?))
    } else {
        Ok(constr.clone())
    }
}
