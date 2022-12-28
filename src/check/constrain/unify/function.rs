use std::cmp::max;
use std::collections::HashSet;

use itertools::{EitherOrBoth, enumerate, Itertools};

use crate::check::constrain::constraint::Constraint;
use crate::check::constrain::constraint::expected::Expect::{Access, Field, Function, Type};
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::constraint::iterator::Constraints;
use crate::check::constrain::Unified;
use crate::check::constrain::unify::finished::Finished;
use crate::check::constrain::unify::link::{reinsert, unify_link};
use crate::check::context::{Context, LookupClass};
use crate::check::context::arg::FunctionArg;
use crate::check::context::clss::{GetField, GetFun};
use crate::check::name::{Empty, Name, Union};
use crate::check::name::name_variant::NameVariant;
use crate::check::name::string_name::StringName;
use crate::check::result::TypeErr;
use crate::common::delimit::comma_delm;
use crate::common::position::Position;

pub fn unify_function(
    constraint: &Constraint,
    constraints: &mut Constraints,
    finished: &mut Finished,
    ctx: &Context,
    total: usize,
) -> Unified {
    let (left, right) = (&constraint.left, &constraint.right);
    match (&left.expect, &right.expect) {
        (Function { args, .. }, Type { name }) | (Type { name }, Function { args, .. }) => {
            let arguments_union: Vec<Vec<Name>> = name
                .names
                .iter()
                .cloned()
                .map(|n| match n.variant {
                    NameVariant::Fun(arguments, _) => Ok(arguments),
                    other => {
                        let msg = format!("A '{other}' does not take arguments");
                        Err(vec![TypeErr::new(right.pos, &msg)])
                    }
                })
                .collect::<Result<_, _>>()?;

            let mut count = 0;
            for arguments in arguments_union {
                for possible in arguments.iter().zip_longest(args.iter()) {
                    match possible {
                        EitherOrBoth::Both(arg, expected) => {
                            count += 1;
                            let arg_ty = Expected::new(left.pos, &Type { name: arg.clone() });
                            constraints.push("anonymous function argument", &arg_ty, expected)
                        }
                        EitherOrBoth::Left(_) | EitherOrBoth::Right(_) => {
                            let msg = format!(
                                "{} arguments given to function '{}', which takes {} arguments",
                                args.len(),
                                &left.expect,
                                arguments.len()
                            );
                            return Err(vec![TypeErr::new(left.pos, &msg)]);
                        }
                    }
                }
            }

            unify_link(constraints, finished, ctx, total + count)
        }

        (Access { entity, name }, _) =>
            access(constraints, finished, ctx, constraint, entity, name, true, total),
        (_, Access { entity, name }) =>
            access(constraints, finished, ctx, constraint, entity, name, false, total),

        (l_exp, r_exp) => {
            let msg = format!("Unifying function: Expected a '{l_exp}', was a '{r_exp}'");
            Err(vec![TypeErr::new(left.pos, &msg)])
        }
    }
}

fn access(
    constraints: &mut Constraints,
    finished: &mut Finished,
    ctx: &Context,
    constraint: &Constraint,
    entity: &Expected,
    name: &Box<Expected>,
    access_left: bool,
    total: usize,
) -> Unified {
    let (left, right) = (&constraint.left, &constraint.right);
    let (left, right) = if access_left { (left, right) } else { (right, left) };

    if let Type { name: entity_name } = &entity.expect {
        match &name.expect {
            Field { name } => {
                field_access(constraints, finished, ctx, entity_name, name, left, right, total)
            }
            Function { name, args } => {
                function_access(constraints, finished, ctx, entity_name, name, args, left, right, total)
            }
            _ => {
                reinsert(constraints, constraint, total)?;
                unify_link(constraints, finished, ctx, total)
            }
        }
    } else {
        reinsert(constraints, constraint, total)?;
        unify_link(constraints, finished, ctx, total)
    }
}

#[allow(clippy::too_many_arguments)]
fn field_access(
    constraints: &mut Constraints,
    finished: &mut Finished,
    ctx: &Context,
    entity_name: &Name,
    name: &str,
    accessed: &Expected,
    other: &Expected,
    total: usize,
) -> Unified {
    let mut pushed = 0;
    let fields = ctx.class(entity_name, accessed.pos)?.field(name, ctx, accessed.pos)?;
    for field in fields.union {
        let field_ty_exp = Expected::new(accessed.pos, &Type { name: field.ty });
        constraints.push("field access", other, &field_ty_exp);
        pushed += 1;
    }

    unify_link(constraints, finished, ctx, total + pushed)
}

#[allow(clippy::too_many_arguments)]
fn function_access(
    constraints: &mut Constraints,
    finished: &mut Finished,
    ctx: &Context,
    entity_name: &Name,
    name: &StringName,
    args: &[Expected],
    accessed: &Expected,
    other: &Expected,
    total: usize,
) -> Unified {
    let class = ctx.class(entity_name, accessed.pos)?;
    let function_union = class.fun(name, ctx, accessed.pos)?;

    let mut pushed = 0;
    for function in &function_union.union {
        let fun_ty_exp = Expected::new(accessed.pos, &Type { name: function.ret_ty.clone() });
        constraints.push("function access", other, &fun_ty_exp);
        pushed += 1;
    }

    let largest = function_union.union.iter().fold(0, |m, f| max(m, f.arguments.len()));
    let mut possible_args: Vec<HashSet<FunctionArg>> = vec![HashSet::new(); largest];
    for fun in function_union.union {
        for (i, arg) in enumerate(fun.arguments) {
            possible_args[i].insert(arg);
        }
    }

    let (mut constr, added) = unify_fun_arg(&possible_args, args, constraints, accessed.pos)?;
    unify_link(&mut constr, finished, ctx, total + added + pushed)
}

fn unify_fun_arg(
    ctx_f_args: &[HashSet<FunctionArg>],
    args: &[Expected],
    constr: &Constraints,
    pos: Position,
) -> Unified<(Constraints, usize)> {
    let mut constr = constr.clone();
    let mut added = 0;

    for either_or_both in ctx_f_args.iter().zip_longest(args.iter()) {
        match either_or_both {
            EitherOrBoth::Both(ctx_f_arg, expected) => {
                let names = ctx_f_arg
                    .iter()
                    .map(|f_arg| {
                        f_arg.ty.clone().ok_or({
                            vec![TypeErr::new(pos, &format!("Argument '{f_arg}' has no type"))]
                        })
                    })
                    .collect::<Result<Vec<Name>, _>>()?;

                let name = names.iter().fold(Name::empty(), |name, f_name| name.union(f_name));
                let ctx_arg_ty = Expected::new(expected.pos, &Type { name });
                constr.push("function argument", &ctx_arg_ty, expected);
                added += 1;
            }
            EitherOrBoth::Left(fun_arg) if !fun_arg.iter().any(|a| !a.has_default) => {
                let msg = format!("Expected argument for '{}'", comma_delm(fun_arg));
                return Err(vec![TypeErr::new(pos, &msg)]);
            }
            EitherOrBoth::Right(_) => {
                let msg = format!("Function takes only {} {}", ctx_f_args.len(), if ctx_f_args.len() == 1 { "argument" } else { "arguments" });
                return Err(vec![TypeErr::new(pos, &msg)]);
            }
            _ => {}
        }
    }

    Ok((constr, added))
}
