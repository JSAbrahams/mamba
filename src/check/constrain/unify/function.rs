use itertools::{EitherOrBoth, Itertools};

use crate::check::constrain::constraint::Constraint;
use crate::check::constrain::constraint::expected::Expect::{Access, Field, Function, Type};
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::constraint::iterator::Constraints;
use crate::check::constrain::Unified;
use crate::check::constrain::unify::finished::Finished;
use crate::check::constrain::unify::link::{reinsert, unify_link};
use crate::check::constrain::unify::ty::unify_type_message;
use crate::check::context::{Context, LookupClass};
use crate::check::context::arg::{FunctionArg, SELF};
use crate::check::context::clss::{GetField, GetFun};
use crate::check::context::function::python::STR;
use crate::check::name::{Empty, Mutable, Name, TupleCallable};
use crate::check::name::string_name::StringName;
use crate::check::name::true_name::TrueName;
use crate::check::result::TypeErr;
use crate::common::delimit::comma_delm;
use crate::common::position::Position;
use crate::common::result::WithCause;

pub fn unify_function(
    constraint: &Constraint,
    constraints: &mut Constraints,
    finished: &mut Finished,
    ctx: &Context,
    total: usize,
) -> Unified {
    let (left, right) = (&constraint.parent, &constraint.child);
    match (&left.expect, &right.expect) {
        (Function { args, .. }, Type { name }) | (Type { name }, Function { args, .. }) => {
            let arguments_union: Vec<Vec<Name>> = name
                .names
                .iter()
                .cloned()
                .map(|n| n.args(right.pos))
                .collect::<Result<_, _>>()?;

            let mut count = 0;
            for arguments in arguments_union {
                for possible in arguments.iter().zip_longest(args.iter()) {
                    match possible {
                        EitherOrBoth::Both(arg, expected) => {
                            count += 1;
                            let arg_ty = Expected::new(left.pos, &Type { name: arg.clone() });
                            let constr = Constraint::new("anonymous function argument", &arg_ty, expected);
                            constraints.push_front(&constr.propagate(false));
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

        _ => Err(unify_type_message("access", &constraint.msg, left, right))
    }
}

#[allow(clippy::too_many_arguments)]
fn access(
    constraints: &mut Constraints,
    finished: &mut Finished,
    ctx: &Context,
    constraint: &Constraint,
    entity: &Expected,
    name: &Expected,
    access_left: bool,
    total: usize,
) -> Unified {
    let (left, right) = (&constraint.parent, &constraint.child);
    let (left, right) = if access_left { (left, right) } else { (right, left) };

    if let Type { name: entity_name } = &entity.expect {
        match &name.expect {
            Field { name } => {
                field_access(constraints, finished, ctx, entity_name, name, left, right, &constraint.msg, total)
            }
            Function { name, args } => {
                function_access(constraints, finished, ctx, entity_name, name, args, left, right, &constraint.msg, total)
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
    msg: &str,
    total: usize,
) -> Unified {
    if entity_name.is_empty() {
        let msg = format!("{entity_name} does not define {name}");
        return Err(vec![TypeErr::new(accessed.pos, &msg)]);
    }

    let mut pushed = 0;
    for entity_name in &entity_name.names {
        let field = ctx.class(entity_name, accessed.pos)
            .map_err(|errs| access_class_cause(&errs, other, accessed, entity_name, msg))?
            .field(name, accessed.pos)
            .map_err(|errs| access_field_cause(&errs, other, entity_name, name, msg))?;

        let field_ty_exp = Expected::new(accessed.pos, &Type { name: field.ty });
        let constr = Constraint::new("field access", &field_ty_exp, other);
        constraints.push_front(&constr.propagate(false));
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
    msg: &str,
    total: usize,
) -> Unified {
    if entity_name.is_empty() {
        let msg = format!("{entity_name} does not define {name}");
        return Err(vec![TypeErr::new(accessed.pos, &msg)]);
    }

    let mut pushed = 0;
    for entity_name in &entity_name.names {
        let class = ctx.class(entity_name, accessed.pos)
            .map_err(|errs| access_class_cause(&errs, other, accessed, entity_name, msg))?;
        let fun = class.fun(name, accessed.pos)
            .map_err(|errs| access_fun_cause(&errs, other, entity_name, name, args, msg))?;

        let fun_ty_exp = Expected::new(accessed.pos, &Type { name: fun.ret_ty.clone() });
        let constr = Constraint::new("function access", other, &fun_ty_exp);
        constraints.push_front(&constr.propagate(false));
        pushed += 1;

        pushed += unify_fun_arg(entity_name, name, &fun.arguments, args, constraints, accessed.pos)?;
    }

    unify_link(constraints, finished, ctx, total + pushed)
}

fn unify_fun_arg(
    entity_name: &TrueName,
    name: &StringName,
    ctx_f_args: &[FunctionArg],
    args: &[Expected],
    constr: &mut Constraints,
    pos: Position,
) -> Unified<usize> {
    let mut added = 0;

    for either_or_both in ctx_f_args.iter().zip_longest(args.iter()) {
        match either_or_both {
            EitherOrBoth::Both(ctx_f_arg, expected) => {
                let Some(arg_name) = ctx_f_arg.ty.clone() else {
                    let msg = format!("Argument '{ctx_f_arg}' in context has no type");
                    return Err(vec![TypeErr::new(pos, &msg)]);
                };
                let ctx_arg_ty = Expected::new(expected.pos, &Type { name: arg_name });

                // self is special, because self is equal to entity name
                let expected = if ctx_f_arg.name == SELF {
                    if let Type { name } = &expected.expect {
                        let entity_name = if ctx_f_arg.mutable { entity_name.as_mutable() } else { entity_name.clone() };
                        Expected::new(expected.pos, &Type { name: name.as_name(&entity_name, pos)? })
                    } else { expected.clone() }
                } else { expected.clone() };

                if let Ok(Ok(tuple_union)) = expected.ty().map(|name| name.elements(expected.pos)) {
                    // exception for tuple, since that is variable generic count
                    if name == &StringName::from(STR) {
                        for tuple_element in tuple_union.iter().flatten() {
                            let expected = Expected::new(expected.pos, &Type { name: tuple_element.clone() });
                            let msg = format!("tuple element define {STR}");
                            let stringy = Constraint::stringy(&msg, &expected);
                            constr.push_back(&stringy.propagate(false));
                        }
                    } else {
                        let msg = format!("function arg in {name}: {}", ctx_f_arg.name);
                        let constraint = Constraint::new(&msg, &ctx_arg_ty, &expected);
                        constr.push_front(&constraint.propagate(false));
                    }
                } else {
                    let msg = format!("function arg in {name}: {}", ctx_f_arg.name);
                    let constraint = Constraint::new(&msg, &ctx_arg_ty, &expected);
                    constr.push_front(&constraint.propagate(false));
                }

                added += 1;
            }
            EitherOrBoth::Left(fun_arg) if !fun_arg.has_default => {
                let msg = format!("Expected argument for '{fun_arg}' in method {name} of {entity_name}");
                return Err(vec![TypeErr::new(pos, &msg)]);
            }
            EitherOrBoth::Right(_) => {
                let msg = format!("Method {name} of {entity_name} takes only {} {}, received {}: {}",
                                  ctx_f_args.len(),
                                  if ctx_f_args.len() == 1 { "argument" } else { "arguments" },
                                  args.len(),
                                  comma_delm(args));
                return Err(vec![TypeErr::new(pos, &msg)]);
            }
            _ => {}
        }
    }

    Ok(added)
}

fn access_class_cause(errs: &[TypeErr], other: &Expected, actual: &Expected, entity_name: &TrueName, cause: &str) -> Vec<TypeErr> {
    let msg = format!("In {cause}, we expect {entity_name}, was {actual}");
    access_cause(errs, other, &msg, cause)
}

fn access_field_cause(errs: &[TypeErr], other: &Expected, entity_name: &TrueName, field_name: &str, cause: &str) -> Vec<TypeErr> {
    let msg = format!("We expect {other}, but {entity_name} does not define {field_name}");
    access_cause(errs, other, &msg, cause)
}

fn access_fun_cause(errs: &[TypeErr], other: &Expected, entity_name: &TrueName, fun_name: &StringName, args: &[Expected], cause: &str) -> Vec<TypeErr> {
    let args: Vec<Expected> = args.iter().map(|a| a.and_or_a(false)).collect();
    let msg = format!("We expect {other}, but {entity_name} does not define {fun_name}({})", comma_delm(args));
    access_cause(errs, other, &msg, cause)
}

fn access_cause(errs: &[TypeErr], other: &Expected, msg: &str, cause: &str) -> Vec<TypeErr> {
    errs.iter().map(|err| {
        // flip messages
        let err = if let Some(pos) = err.pos {
            TypeErr::new(pos, msg)
        } else {
            TypeErr::new_no_pos(msg)
        };

        err.with_cause(&format!("In {cause}"), other.pos)
    }).collect()
}
