use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::ops::Deref;

use crate::parser::ast::{Node, AST};
use crate::type_checker::context::function_arg::generic::argument_name;
use crate::type_checker::context::ty::concrete;
use crate::type_checker::context::type_name::actual::ActualTypeName;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::context::{function_arg, Context};
use crate::type_checker::environment::expression_type::ExpressionType;
use crate::type_checker::environment::infer_type::InferType;
use crate::type_checker::environment::name::{match_name, Identifier};
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::{TypeErr, TypeResult};
use crate::type_checker::util::comma_delimited;

pub fn infer_assign(ast: &AST, env: &Environment, ctx: &Context) -> InferResult {
    // TODO check body of function definition
    // TODO if self, use state to determine type of current class
    match &ast.node {
        Node::Undefined => Ok((
            InferType::from(&ctx.lookup(&TypeName::from(concrete::NONE), &ast.pos)?),
            env.clone()
        )),
        Node::Id { lit } => Ok((InferType::from(&env.lookup(lit, &ast.pos)?), env.clone())),
        Node::IdType { .. } => Ok((InferType::new(), env.clone())),
        Node::Reassign { left, right } => {
            let identifier = Identifier::try_from(left.deref())?;
            let (right_ty, env) = infer(right, &env, ctx)?;
            let right_expr = right_ty.expr_ty(&right.pos)?;

            let mut env = env.clone();
            let matched = match_name(&identifier, &right_expr, &env, &right.pos)?;
            for (id, (new_mutable, expr_ty)) in matched {
                let (mutable, left_expr) = env.lookup_indirect(&id, &left.pos)?;
                if !mutable {
                    let msg = format!("Attempting to assign to immutable variable {}", id);
                    return Err(vec![TypeErr::new(&left.pos, &msg)]);
                }

                if left_expr != expr_ty {
                    let msg = format!("Expected {}, but was {}", left_expr, right_expr);
                    return Err(vec![TypeErr::new(&ast.pos, &msg)]);
                }

                env.insert(&id, new_mutable, &expr_ty);
            }

            Ok((InferType::new().add_raises(&right_ty), env))
        }
        // TODO use forward and private
        Node::VariableDef { id_maybe_type, expression, .. } => match &id_maybe_type.node {
            // TODO Check whether mutable
            // TODO use system for tuples of ids
            Node::IdType { _type, id, mutable } => {
                let identifier = Identifier::try_from(id.deref())?;

                let (ty, mut env) = match (_type, expression) {
                    (Some(ty_name), Some(expr)) => {
                        let expr_ty =
                            ctx.lookup(&TypeName::try_from(ty_name.deref())?, &ty_name.pos)?;
                        let (other_ty, env) = infer(expr, env, ctx)?;
                        if expr_ty != other_ty.expr_ty(&id_maybe_type.pos)? {
                            let msg = "Expression type does not match annotated type";
                            return Err(vec![TypeErr::new(&expr.pos, msg)]);
                        }

                        (other_ty, env)
                    }
                    (None, Some(expr)) => infer(expr, env, ctx)?,
                    (Some(ty_name), None) => (
                        InferType::from(
                            &ctx.lookup(&TypeName::try_from(ty_name.deref())?, &ty_name.pos)?
                        ),
                        env.clone()
                    ),
                    (None, None) => return Err(vec![TypeErr::new(&ast.pos, "Cannot infer type")])
                };

                let expr_ty = ty.expr_ty(&ast.pos)?;
                let matched = match_name(&identifier, &expr_ty, &env, &ast.pos)?;
                for (id, (inner_mut, expr_ty)) in matched {
                    env.insert(id.as_str(), *mutable || inner_mut, &expr_ty);
                }

                Ok((InferType::new(), env))
            }
            _ => Err(vec![TypeErr::new(&ast.pos, "Expected identifier")])
        },

        Node::FunArg { .. } =>
            Err(vec![TypeErr::new(&ast.pos, "Function argument cannot be top level")]),
        Node::FunDef { fun_args, ret_ty, raises, body, .. } => {
            // TODO use pure
            // TODO add functions to environment
            let mut env_with_args = env.clone();
            for (name, (mutable, expr_ty)) in arg_types(fun_args, env, ctx)? {
                env_with_args.insert(&name, mutable, &expr_ty);
            }

            let raises: HashSet<ActualTypeName> = raises
                .iter()
                .map(|raise| ActualTypeName::try_from(raise))
                .collect::<Result<_, _>>()?;
            let ret_ty = match ret_ty {
                Some(ret_ty) => Some(TypeName::try_from(ret_ty.deref())?),
                None => None
            };

            if let Some(body) = body {
                let (body_ty, _) = infer(body, &env_with_args, ctx)?;
                let mut boy_raises_not_in_signature = body_ty.raises.clone();
                boy_raises_not_in_signature.retain(|f| !raises.contains(f));
                if !boy_raises_not_in_signature.is_empty() {
                    return Err(vec![TypeErr::new(
                        &ast.pos,
                        &format!(
                            "Body raises the following which were not mentioned in the signature: \
                             [{}]",
                            comma_delimited(boy_raises_not_in_signature)
                        )
                    )]);
                }

                match ret_ty {
                    Some(ret_ty) =>
                        if body_ty.is_stmt() {
                            Err(vec![TypeErr::new(
                                &ast.pos,
                                &format!("body must have type {}, but was statement", ret_ty)
                            )])
                        } else {
                            // TODO allow return type of be nullable even if body is not
                            let body_ret_name = TypeName::from(&body_ty.expr_ty(&ast.pos)?);
                            // TODO if return type empty and return empty to body
                            if ret_ty.is_superset(&body_ret_name) {
                                Ok((
                                    InferType::from(&ctx.lookup(&ret_ty, &ast.pos)?)
                                        .union_raises(&raises),
                                    env.clone()
                                ))
                            } else {
                                Err(vec![TypeErr::new(
                                    &ast.pos,
                                    &format!(
                                        "Body must have return type {}, was {}",
                                        ret_ty, body_ret_name
                                    )
                                )])
                            }
                        },
                    None => Ok((InferType::new().union_raises(&raises), env.clone()))
                }
            } else {
                Ok((InferType::new().union_raises(&raises), env.clone()))
            }
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected variable manipulation")])
    }
}

pub fn arg_types(
    args: &Vec<AST>,
    env: &Environment,
    ctx: &Context
) -> TypeResult<HashMap<String, (bool, ExpressionType)>> {
    let mut arg_types = HashMap::new();
    for arg in args {
        // TODO do something with vararg
        let (id, mutable, _type, default) = match &arg.node {
            Node::FunArg { id_maybe_type, default, .. } => match &id_maybe_type.node {
                Node::IdType { id, mutable, _type } => (id, mutable, _type, default),
                _ =>
                    return Err(vec![TypeErr::new(
                        &id_maybe_type.pos,
                        "Expected identifier with type"
                    )]),
            },
            _ => return Err(vec![TypeErr::new(&arg.pos, "Expected function argument")])
        };

        let lit = argument_name(id)?;
        let default_ty = match default {
            Some(default) => Some(infer(default, env, ctx)?.0.expr_ty(&id.pos)?),
            None => None
        };

        // TODO if return type is none, then body should not return anything
        // TODO if op overloading, return type Bool or class even if not specified
        if let Some(_type) = _type {
            let arg_ty_name = TypeName::try_from(_type.deref())?;
            if let Some(default_ty) = default_ty {
                if arg_ty_name == TypeName::from(&default_ty) {
                    let expr_ty = ctx.lookup(&arg_ty_name, &_type.pos)?;
                    arg_types.insert(lit.clone(), (*mutable, expr_ty));
                } else {
                    let msg = format!(
                        "default type {} does not match argument type {}",
                        default_ty, arg_ty_name
                    );
                    return Err(vec![TypeErr::new(&arg.pos, &msg)]);
                }
            } else {
                arg_types.insert(lit.clone(), (*mutable, ctx.lookup(&arg_ty_name, &_type.pos)?));
            }
        } else {
            if &lit == function_arg::concrete::SELF {
                // TODO get actual type of self from Context in case self is child of class
                let class = env
                    .state
                    .in_class
                    .clone()
                    .ok_or(vec![TypeErr::new(&arg.pos, "self cannot be outside class")])?;
                arg_types.insert(
                    lit.clone(),
                    (*mutable, ctx.lookup(&TypeName::from(&class), &arg.pos)?)
                );
            } else if let Some(default_ty) = default_ty {
                arg_types.insert(lit.clone(), (*mutable, default_ty));
            } else {
                let msg = format!("Cannot derive type of {}", lit);
                return Err(vec![TypeErr::new(&arg.pos, &msg)]);
            }
        }
    }

    Ok(arg_types)
}
