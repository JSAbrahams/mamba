use crate::common::position::Position;
use crate::parser::ast::Node::Bool;
use crate::parser::ast::AST;
use crate::type_checker::constraints::cons::Expect::*;
use crate::type_checker::constraints::cons::{Constraints, Expected};
use crate::type_checker::context::function;
use crate::type_checker::context::function_arg::concrete::FunctionArg;
use crate::type_checker::context::Context;
use crate::type_checker::type_name::TypeName;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use itertools::{EitherOrBoth, Itertools};
use std::collections::HashSet;

/// Empties out constraints and puts them in a substituted list.
pub fn unify_link(
    constr: &Constraints,
    sub: &Constraints,
    ctx: &Context
) -> TypeResult<Constraints> {
    let mut constraints = constr.clone();
    let mut sub = sub.clone();

    while let Some(constraint) = constraints.constraints.pop() {
        let unified = match (&constraint.0.expect, &constraint.1.expect) {
            (ExpressionAny, ExpressionAny) | (Truthy, Truthy) | (RaisesAny, RaisesAny) =>
                Ok(Constraints::from(&constraint)),

            (Expression { .. }, ExpressionAny) | (ExpressionAny, Expression { .. }) =>
                Ok(Constraints::from(&constraint)),
            (Type { .. }, ExpressionAny) | (ExpressionAny, Type { .. }) =>
                Ok(Constraints::from(&constraint)),

            (Expression { .. }, Expression { .. }) => unify_link(
                &substitute(&constraint.0, &constraint.1, &constraints)?,
                &unify_link(
                    &Constraints::from(&constraint),
                    &substitute(&constraint.0, &constraint.1, &sub)?.add_constraint(&constraint),
                    ctx
                )?,
                ctx
            ),

            (Type { type_name: left }, Type { type_name: right }) if left == right => {
                ctx.lookup(left, &constraint.0.pos)?;
                Ok(Constraints::from(&constraint))
            }
            (Type { type_name: left }, Type { type_name: right }) => Err(vec![TypeErr::new(
                &Position::default(),
                &format!("Types not equal: {}, {}", left, right)
            )]),

            (Type { .. }, Expression { .. }) | (Expression { .. }, Type { .. }) => unify_link(
                &substitute(&constraint.0, &constraint.1, constr)?,
                &substitute(&constraint.0, &constraint.1, &sub)?.add_constraint(&constraint),
                ctx
            ),

            (Truthy, Expression { ast: AST { node: Bool { .. }, .. } })
            | (Expression { ast: AST { node: Bool { .. }, .. } }, Truthy) =>
                Ok(Constraints::from(&constraint)),

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

                let mut constr = constraints.clone();
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
                                constr = constr.add(
                                    &expected,
                                    &Expected::new(&expected.pos, &Type { type_name })
                                )
                            }
                            EitherOrBoth::Left(fun_arg) if !fun_arg.has_default => unimplemented!(),
                            EitherOrBoth::Right(expected) =>
                                return Err(vec![TypeErr::new(
                                    &expected.pos,
                                    "Unexpected argument"
                                )]),
                            _ => {}
                        }
                    }
                }

                unify_link(&constraints, &sub, ctx)
            }

            _ => panic!(
                "Unexpected: {}={} : {:?} == {:?}",
                constraint.0.pos, constraint.1.pos, constraint.1.expect, constraint.1.expect
            )
        }?;

        sub.append(&unified);
    }

    Ok(sub)
}

fn substitute(
    old: &Expected,
    new: &Expected,
    constraints: &Constraints
) -> TypeResult<Constraints> {
    let mut constraints = constraints.clone();
    if let Some(constraint) = constraints.constraints.pop() {
        let (left, right) = (constraint.0, constraint.1);
        let left = if &left == old { Expected::new(&left.pos, &new.expect) } else { left };
        let right = if &right == old { Expected::new(&right.pos, &new.expect) } else { right };
        let mut unified = Constraints::new().add(&left, &right);
        Ok(unified.append(&substitute(old, new, &constraints)?))
    } else {
        Ok(constraints.clone())
    }
}
