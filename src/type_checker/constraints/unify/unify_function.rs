use crate::common::position::Position;
use crate::type_checker::checker_result::TypeErr;
use crate::type_checker::constraints::constraint::expected::Expect::{Access, Field, Function,
                                                                     Statement, Type};
use crate::type_checker::constraints::constraint::expected::Expected;
use crate::type_checker::constraints::constraint::iterator::Constraints;
use crate::type_checker::constraints::constraint::Constraint;
use crate::type_checker::constraints::unify::unify_link::{reinsert, unify_link};
use crate::type_checker::constraints::Unified;
use crate::type_checker::context::function_arg::concrete::FunctionArg;
use crate::type_checker::context::{check_if_parent, Context};
use crate::type_checker::ty_name::TypeName;
use itertools::{EitherOrBoth, Itertools};
use std::collections::HashSet;

pub fn unify_function(
    constr: &Constraint,
    left: &Expected,
    right: &Expected,
    constraints: &mut Constraints,
    ctx: &Context,
    total: usize
) -> Unified {
    match (&left.expect, &right.expect) {
        (Function { args, .. }, Type { type_name }) => {
            let expr_ty = ctx.lookup(type_name, &left.pos)?;
            let functions = expr_ty.anon_fun_params(&left.pos)?;

            let mut count = 0;
            for (f_args, _) in &functions {
                for possible in f_args.iter().zip_longest(args.iter()) {
                    match possible {
                        EitherOrBoth::Both(type_name, expected) => {
                            count += 1;
                            let ty = Type { type_name: type_name.clone() };
                            let right = Expected::new(&left.pos, &ty);
                            constraints.eager_push(expected, &right);
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

            unify_link(constraints, ctx, total + count)
        }

        (Access { entity, name }, _) =>
            if let Type { type_name: entity_name } = &entity.expect {
                match &name.expect {
                    Field { name } => {
                        let field = ctx.lookup(entity_name, &left.pos)?.field(name, &left.pos)?;
                        if field.private {
                            let name = TypeName::new(&field.name, &[]);
                            check_if_parent(
                                &name,
                                &constraints.in_class,
                                entity_name,
                                ctx,
                                &left.pos
                            )?;
                        }
                        let field_ty_exp = if let Some(ty) = field.ty {
                            Expected::new(&left.pos, &Type { type_name: ty.clone() })
                        } else {
                            Expected::new(&left.pos, &Statement)
                        };
                        constraints.eager_push(&right, &field_ty_exp);
                        unify_link(constraints, ctx, total)
                    }
                    Function { name, args } => {
                        let expr_ty = ctx.lookup(entity_name, &left.pos)?;
                        let possible_fun = expr_ty.function(&name, &left.pos)?;

                        for function in &possible_fun {
                            if function.private {
                                let name = TypeName::from(&function.name);
                                check_if_parent(
                                    &name,
                                    &constraints.in_class,
                                    &entity_name,
                                    ctx,
                                    &left.pos
                                )?;
                            }

                            constraints.eager_push(
                                &right,
                                &Expected::new(
                                    &left.pos,
                                    &if let Some(ty) = function.ty() {
                                        Type { type_name: ty }
                                    } else {
                                        Statement
                                    }
                                )
                            );
                        }

                        let possible_args: HashSet<Vec<FunctionArg>> =
                            possible_fun.iter().map(|f| f.arguments.clone()).collect();
                        let (mut constr, added) =
                            unify_fun_arg(&possible_args, &args, &constraints, &left.pos)?;
                        unify_link(&mut constr, ctx, total + added)
                    }
                    _ => {
                        let mut constr = reinsert(constraints, &constr, total)?;
                        unify_link(&mut constr, ctx, total + 1)
                    }
                }
            } else {
                let mut constr = reinsert(constraints, &constr, total)?;
                unify_link(&mut constr, ctx, total + 1)
            },

        other => panic!("function {:?}", other)
    }
}

fn unify_fun_arg(
    possible: &HashSet<Vec<FunctionArg>>,
    args: &[Expected],
    constr: &Constraints,
    pos: &Position
) -> Unified<(Constraints, usize)> {
    let mut constr = constr.clone();
    let mut added = 0;

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

                    added += 1;
                    constr.eager_push(&Expected::new(&expected.pos, &Type { type_name }), &expected)
                }
                EitherOrBoth::Left(fun_arg) if !fun_arg.has_default =>
                    return Err(vec![TypeErr::new(
                        &pos,
                        &format!("Expected argument: expected {}", fun_arg)
                    )]),
                EitherOrBoth::Right(_) =>
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

    Ok((constr, added))
}
