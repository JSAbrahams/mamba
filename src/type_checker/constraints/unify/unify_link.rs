use std::collections::HashSet;
use std::ops::Deref;

use itertools::{EitherOrBoth, Itertools};

use crate::common::position::Position;
use crate::type_checker::constraints::constraint::expected::Expect::*;
use crate::type_checker::constraints::constraint::expected::Expected;
use crate::type_checker::constraints::constraint::{Constraint, Constraints};
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
pub fn unify_link(
    constr: &mut Constraints,
    sub: &Constraints,
    ctx: &Context,
    total: usize
) -> Unified {
    if let Some(constraint) = constr.pop_constr() {
        if constraint.flagged {
            panic!("INFINITE LOOOOOOP")
        }

        let (left, right) = (constraint.left, constraint.right);
        println!(
            "{:width$} [solving {}\\{}{}] {} = {}",
            format!("({}={})", left.pos, right.pos),
            total - constr.constraints.len(),
            total,
            if constraint.flagged { " (flagged)" } else { "" },
            left.expect,
            right.expect,
            width = 30
        );

        match (&left.expect, &right.expect) {
            (ExpressionAny, ExpressionAny)
            | (Truthy, Truthy)
            | (RaisesAny, RaisesAny)
            | (Truthy, ExpressionAny)
            | (ExpressionAny, Truthy)
            | (Type { .. }, ExpressionAny)
            | (ExpressionAny, Type { .. }) => unify_link(constr, sub, ctx, total),

            (Expression { .. }, ExpressionAny) | (ExpressionAny, Expression { .. }) =>
                unify_link(constr, sub, ctx, total),

            (Expression { .. }, Expression { .. })
            | (Expression { .. }, Type { .. })
            | (Expression { .. }, Truthy) => {
                let mut constr = substitute(&left, &right, &constr)?;
                let mut subst = Constraints::from(&Constraint::new(left.clone(), right.clone()));
                subst.append(&substitute(&left, &right, &sub)?);
                unify_link(&mut constr, &subst, ctx, total)
            }

            (Type { .. }, Expression { .. }) | (Truthy, Expression { .. }) => {
                let mut constr = substitute(&right, &left, &constr)?;
                let mut subst = Constraints::from(&Constraint::new(left.clone(), right.clone()));
                subst.append(&substitute(&right, &left, &sub)?);
                unify_link(&mut constr, &subst, ctx, total)
            }

            (Type { type_name: l_name }, Type { type_name: r_name }) if l_name == r_name => {
                // TODO do something with child types
                ctx.lookup(l_name, &left.pos)?;
                unify_link(constr, sub, ctx, total)
            }
            (Type { type_name }, Truthy) | (Truthy, Type { type_name }) => {
                let expr_ty = ctx.lookup(type_name, &left.pos)?;
                expr_ty.fun_args(&TypeName::from(function::python::TRUTHY), &left.pos)?;
                unify_link(constr, sub, ctx, total)
            }
            (Type { type_name: left }, Type { type_name: right }) => Err(vec![TypeErr::new(
                &Position::default(),
                &format!("Types not equal: {}, {}", left, right)
            )]),

            (Type { type_name }, Implements { type_name: f_name, args, ret_ty })
            | (Implements { type_name: f_name, args, ret_ty }, Type { type_name }) => {
                let expr_ty = ctx.lookup(type_name, &left.pos)?;
                let f_name = if f_name == type_name {
                    ctx.lookup(f_name, &left.pos)?;
                    TypeName::from(function::concrete::INIT)
                } else {
                    f_name.clone()
                };

                let possible = expr_ty.fun_args(&f_name, &left.pos)?;
                let mut constr = unify_fun_arg(possible, args, constr, &left.pos)?;

                if let Some(ret_ty) = ret_ty {
                    for type_name in expr_ty.fun_ret_ty(&f_name, &left.pos)? {
                        let right = Expected::new(&left.pos, &Type { type_name });
                        constr.push(ret_ty, &right);
                    }
                }

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

            (Nullable { expect }, _) => {
                constr.push(&Expected::new(&left.pos, expect.deref()), &right);
                unify_link(constr, sub, ctx, total)
            }
            (_, Nullable { expect }) => {
                constr.push(&left, &Expected::new(&right.pos, expect.deref()));
                unify_link(constr, sub, ctx, total)
            }

            (Type { type_name }, Function { name, args, ret_ty })
            | (Function { name, args, ret_ty }, Type { type_name }) => {
                let expr_ty = ctx.lookup(type_name, &left.pos)?;
                let functions = expr_ty.anon_fun_params(&left.pos)?;

                for (f_args, f_ret_ty) in &functions {
                    for possible in f_args.into_iter().zip_longest(args.iter()) {
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

                    if let Some(ret_ty) = ret_ty {
                        let expected =
                            Expected::new(&left.pos, &Type { type_name: f_ret_ty.clone() });
                        constr.push(ret_ty, &expected);
                    }
                }

                unify_link(constr, sub, ctx, total)
            }

            (Type { type_name }, HasField { name }) => {
                let expr_ty = ctx.lookup(type_name, &left.pos)?;
                let field_ty = expr_ty.field(name, &right.pos)?.ty()?;
                let field_type = Expected::new(&right.pos, &Type { type_name: field_ty });

                let mut constr = substitute(&right, &field_type, constr)?;
                unify_link(&mut constr, sub, ctx, total)
            }
            (HasField { name }, Type { type_name }) => {
                let expr_ty = ctx.lookup(type_name, &right.pos)?;
                let field_ty = expr_ty.field(name, &left.pos)?.ty()?;
                let field_type = Expected::new(&left.pos, &Type { type_name: field_ty });

                let mut constr = substitute(&left, &field_type, constr)?;
                unify_link(&mut constr, sub, ctx, total)
            }

            (HasField { name: l_name }, HasField { name: r_name }) =>
                if l_name == r_name {
                    unify_link(constr, sub, ctx, total)
                } else {
                    // This should in theory never occur however
                    let msg = format!("Field access differs: {} != {}", left.expect, right.expect);
                    Err(vec![TypeErr::new(&left.pos, &msg)])
                },

            _ => {
                // Defer to later point
                println!(
                    "{:width$} [reinserting {}\\{}] {} = {}",
                    format!("({}={})", left.pos, right.pos),
                    total - constr.constraints.len(),
                    total,
                    left.expect,
                    right.expect,
                    width = 32
                );

                constr.reinsert(&Constraint::new(left, right))?;
                unify_link(constr, sub, ctx, total)
            }
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
