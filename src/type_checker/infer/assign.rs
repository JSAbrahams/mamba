use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::ops::Deref;

use crate::parser::ast::{Node, AST};
use crate::type_checker::context::ty::concrete;
use crate::type_checker::context::type_name::actual::ActualTypeName;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::context::Context;
use crate::type_checker::environment::expression_type::ExpressionType;
use crate::type_checker::environment::infer_type::InferType;
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::{TypeErr, TypeResult};
use crate::type_checker::util::comma_delimited;

pub fn infer_assign(ast: &AST, env: &Environment, ctx: &Context, state: &State) -> InferResult {
    // TODO check body of function definition
    // TODO if self, use state to determine type of current class
    match &ast.node {
        Node::Undefined => Ok((
            InferType::from(&ctx.lookup(&TypeName::from(concrete::NONE), &ast.pos)?),
            env.clone()
        )),
        Node::Id { lit } => Ok((InferType::from(&env.lookup(lit, &ast.pos)?), env.clone())),
        Node::Reassign { left, right } => {
            let (left_ty, env) = infer(left, env, ctx, state)?;
            let (right_ty, env) = infer(right, &env, ctx, state)?;

            let left_expr = left_ty.expr_ty(&ast.pos)?;
            // TODO reevaluate how we deal with mutable (should this be expression level?)
            let right_expr = right_ty.expr_ty(&ast.pos)?;
            if left_expr == right_expr {
                Ok((InferType::new().add_raises(&left_ty).add_raises(&right_ty), env))
            } else {
                Err(vec![TypeErr::new(
                    &ast.pos,
                    &format!("Types must be equal, should be {}, was {}", left_expr, right_expr)
                )])
            }
        }
        // TODO use forward and private
        Node::VariableDef { id_maybe_type, expression, .. } => match &id_maybe_type.node {
            // TODO Check whether mutable
            // TODO use system for tuples of ids
            Node::IdType { _type, id, mutable } => {
                let id = match &id.node {
                    Node::Id { lit } => lit.clone(),
                    _ => return Err(vec![TypeErr::new(&id.pos, "Expected identifier")])
                };

                let (ty, mut env) = match (_type, expression) {
                    (Some(ty_name), Some(expr)) => {
                        let infer_type =
                            ctx.lookup(&TypeName::try_from(ty_name.deref())?, &ty_name.pos)?;
                        let (other_ty, env) = infer(expr, env, ctx, state)?;
                        if infer_type != other_ty.expr_ty(&id_maybe_type.pos)? {
                            return Err(vec![TypeErr::new(
                                &expr.pos,
                                "Expression type does not match annotated type"
                            )]);
                        }

                        (other_ty, env)
                    }
                    (None, Some(expr)) => infer(expr, env, ctx, state)?,
                    (Some(ty_name), None) => (
                        InferType::from(
                            &ctx.lookup(&TypeName::try_from(ty_name.deref())?, &ty_name.pos)?
                        ),
                        env.clone()
                    ),
                    (None, None) => return Err(vec![TypeErr::new(&ast.pos, "Cannot infer type")])
                };

                env.insert(id.as_str(), *mutable, &ty.expr_ty(&ast.pos)?);
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
            for (name, (mutable, expr_ty)) in arg_types(fun_args, env, ctx, state)? {
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
                let (body_ty, _) = infer(body, &env_with_args, ctx, state)?;
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
                                &format!("body must have type {}", ret_ty)
                            )])
                        } else {
                            // TODO allow return type of be nullable even if body is not
                            let body_ret_name = TypeName::from(&body_ty.expr_ty(&ast.pos)?);
                            if body_ret_name == ret_ty {
                                Ok((InferType::from(&ctx.lookup(&ret_ty, &ast.pos)?), env.clone()))
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
                    None => Ok((InferType::new(), env.clone()))
                }
            } else {
                Ok((InferType::new(), env.clone()))
            }
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected variable manipulation")])
    }
}

pub fn arg_types(
    args: &Vec<AST>,
    env: &Environment,
    ctx: &Context,
    state: &State
) -> TypeResult<HashMap<String, (bool, ExpressionType)>> {
    let mut arg_types = HashMap::new();
    for arg in args {
        // TODO do something with vararg
        match &arg.node {
            Node::FunArg { vararg, id_maybe_type, default } => match &id_maybe_type.node {
                Node::IdType { id, mutable, _type } => match &id.node {
                    Node::Id { lit } => {
                        let default_ty = match default {
                            Some(default) =>
                                Some(infer(default, env, ctx, state)?.0.expr_ty(&id.pos)?),
                            None => None
                        };

                        if let Some(_type) = _type {
                            let arg_ty_name = TypeName::try_from(_type.deref())?;
                            if let Some(default_ty) = default_ty {
                                if arg_ty_name == TypeName::from(&default_ty) {
                                    arg_types.insert(
                                        lit.clone(),
                                        (*mutable, ctx.lookup(&arg_ty_name, &_type.pos)?)
                                    );
                                } else {
                                    return Err(vec![TypeErr::new(
                                        &arg.pos,
                                        &format!(
                                            "default type {} does not match argument type {}",
                                            default_ty, arg_ty_name
                                        )
                                    )]);
                                }
                            } else {
                                arg_types.insert(
                                    lit.clone(),
                                    (*mutable, ctx.lookup(&arg_ty_name, &_type.pos)?)
                                );
                            }
                        } else {
                            if let Some(default_ty) = default_ty {
                                arg_types.insert(lit.clone(), (*mutable, default_ty));
                            } else {
                                return Err(vec![TypeErr::new(
                                    &id_maybe_type.pos,
                                    &format!("Cannot derive type of {}", lit)
                                )]);
                            }
                        }
                    }
                    _ => return Err(vec![TypeErr::new(&id.pos, "Expected identifier")])
                },
                _ =>
                    return Err(vec![TypeErr::new(
                        &id_maybe_type.pos,
                        "Expected identifier with type"
                    )]),
            },
            _ => return Err(vec![TypeErr::new(&arg.pos, "Expected function argument")])
        }
    }

    Ok(arg_types)
}
