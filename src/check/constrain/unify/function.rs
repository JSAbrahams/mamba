use std::collections::HashSet;

use itertools::{EitherOrBoth, Itertools};

use crate::check::constrain::constraint::expected::Expect::{Access, Field, Function, Type};
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::constraint::iterator::Constraints;
use crate::check::constrain::constraint::Constraint;
use crate::check::constrain::unify::link::{reinsert, unify_link};
use crate::check::constrain::Unified;
use crate::check::context::arg::FunctionArg;
use crate::check::context::name::{NameUnion, NameVariant};
use crate::check::context::util::check_is_parent;
use crate::check::context::{Context, LookupClass};
use crate::check::result::TypeErr;
use crate::common::position::Position;

pub fn unify_function(
    constr: &Constraint,
    left: &Expected,
    right: &Expected,
    constraints: &mut Constraints,
    ctx: &Context,
    total: usize
) -> Unified {
    match (&left.expect, &right.expect) {
        (Function { args, .. }, Type { name }) => {
            let arguments_union: Vec<Vec<NameUnion>> = name
                .names()
                .map(|n| match n.variant {
                    NameVariant::Fun(arguments, _) => Ok(arguments),
                    other => {
                        let msg = format!("A '{}' does not take arguments", other);
                        Err(vec![TypeErr::new(&right.pos, &msg)])
                    }
                })
                .collect::<Result<_, _>>()?;

            let mut count = 0;
            for arguments in arguments_union {
                for possible in arguments.iter().zip_longest(args.iter()) {
                    match possible {
                        EitherOrBoth::Both(arg, expected) => {
                            count += 1;
                            let arg_ty = Expected::new(&left.pos, &Type { name: arg.clone() });
                            constraints.push("anonymous function argument", &arg_ty, expected)
                        }
                        EitherOrBoth::Left(_) | EitherOrBoth::Right(_) => {
                            let msg = format!(
                                "{} arguments given to function '{}', which takes {} arguments",
                                args.len(),
                                &left.expect,
                                arguments.len()
                            );
                            return Err(vec![TypeErr::new(&left.pos, &msg)]);
                        }
                    }
                }
            }

            unify_link(constraints, ctx, total + count)
        }

        (Access { entity, name }, _) =>
            if let Type { name: entity_name } = &entity.expect {
                match &name.expect {
                    Field { name } => {
                        let mut pushed = 0;
                        let fields =
                            ctx.class(entity_name, &left.pos)?.field(name, ctx, &left.pos)?;
                        for field in fields.union {
                            if field.private {
                                check_is_parent(
                                    &field.ty,
                                    &constraints.in_class,
                                    entity_name,
                                    ctx,
                                    &left.pos
                                )?;
                            }
                            let field_ty_exp = Expected::new(&left.pos, &Type { name: field.ty });
                            constraints.push("field access", &right, &field_ty_exp);
                            pushed += 1;
                        }
                        unify_link(constraints, ctx, total + pushed)
                    }
                    Function { name, args } => {
                        let class = ctx.class(entity_name, &left.pos)?;
                        let function_union = class.fun(&name, ctx, &left.pos)?;

                        let mut pushed = 0;
                        for function in &function_union.union {
                            if function.private {
                                check_is_parent(
                                    &NameUnion::from(&function.name),
                                    &constraints.in_class,
                                    entity_name,
                                    ctx,
                                    &left.pos
                                )?;
                            }

                            let fun_ty_exp =
                                Expected::new(&left.pos, &Type { name: function.ret_ty.clone() });
                            constraints.push("function access", &right, &fun_ty_exp);
                            pushed += 1;
                        }

                        let possible_args: HashSet<Vec<FunctionArg>> =
                            function_union.union.iter().map(|f| f.arguments.clone()).collect();
                        let (mut constr, added) =
                            unify_fun_arg(&possible_args, &args, &constraints, &left.pos)?;
                        unify_link(&mut constr, ctx, total + added + pushed)
                    }
                    _ => {
                        let mut constr = reinsert(constraints, &constr, total)?;
                        unify_link(&mut constr, ctx, total)
                    }
                }
            } else {
                let mut constr = reinsert(constraints, &constr, total)?;
                unify_link(&mut constr, ctx, total)
            },

        (l_exp, r_exp) => {
            let msg = format!("Expected '{}', found '{}'", l_exp, r_exp);
            Err(vec![TypeErr::new(&left.pos, &msg)])
        }
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
                    let name = &fun_arg.ty.clone().ok_or_else(|| {
                        TypeErr::new(&expected.pos, "Function argument must have type parameters")
                    })?;
                    added += 1;
                    let ty = Expected::new(&expected.pos, &Type { name: name.clone() });
                    constr.push("function argument", &ty, &expected)
                }
                EitherOrBoth::Left(fun_arg) if !fun_arg.has_default =>
                    return Err(vec![TypeErr::new(
                        &pos,
                        &format!("Expected argument: expected {}", fun_arg)
                    )]),
                EitherOrBoth::Right(_) => {
                    let msg = format!("Function takes only {} arguments", f_args.len());
                    return Err(vec![TypeErr::new(&pos, &msg)]);
                }
                _ => {}
            }
        }
    }

    Ok((constr, added))
}
