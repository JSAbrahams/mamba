use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::ops::Deref;

use crate::parser::ast::{Node, AST};
use crate::type_checker::context::function_arg::generic::argument_name;
use crate::type_checker::context::ty::concrete;
use crate::type_checker::context::{function_arg, Context};
use crate::type_checker::environment::name::{match_name, Identifier};
use crate::type_checker::environment::state::StateType::InFunction;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::infer_type::expression::ExpressionType;
use crate::type_checker::infer_type::InferType;
use crate::type_checker::type_name::actual::ActualTypeName;
use crate::type_checker::type_name::TypeName;
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
        Node::ExpressionType { .. } => Ok((InferType::default(), env.clone())),
        Node::Reassign { left, right } => {
            let identifier = Identifier::try_from(left.deref())?;
            let (right_ty, env) = infer(right, &env, ctx)?;
            let right_expr = right_ty.expr_ty(&right.pos)?;

            let mut env = env;
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

            Ok((InferType::default().add_raises(&right_ty), env))
        }
        // TODO use forward and private
        // TODO check if parent already defines variable if relevant
        Node::VariableDef { var, mutable, ty, expression, .. } => {
            let identifier = Identifier::try_from(var.deref())?;

            let (ty, mut env) = match (ty, expression) {
                (Some(ty), Some(expr)) => {
                    let expr_ty = ctx.lookup(&TypeName::try_from(ty.deref())?, &ty.pos)?;
                    let (other_ty, env) = infer(expr, env, ctx)?;
                    if expr_ty != other_ty.expr_ty(&ty.pos)? {
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

            Ok((InferType::default().union_raises(&ty.raises), env))
        }

        Node::FunArg { .. } => Err(vec![TypeErr::new(&ast.pos, "Unexpected function argument")]),
        Node::FunDef { fun_args, ret_ty, raises, body, .. } => {
            // TODO use pure
            let mut inner_env = env.new_state(&env.state.as_state(InFunction));
            arg_types(fun_args, env, ctx)?
                .iter()
                .for_each(|(name, (m, expr_ty))| inner_env.insert(name, *m, expr_ty));

            let raises: HashSet<_> =
                raises.iter().map(ActualTypeName::try_from).collect::<Result<_, _>>()?;
            let ret_ty = match ret_ty {
                Some(ret_ty) => Some(TypeName::try_from(ret_ty.deref())?),
                None => None
            };

            if let Some(body) = body {
                let (body_ty, _) = infer(body, &inner_env, ctx)?;
                let raises_not_in_signature: HashSet<_> =
                    body_ty.raises.difference(&raises).collect();
                if !raises_not_in_signature.is_empty() {
                    let raised = comma_delimited(raises_not_in_signature);
                    let msg = format!(
                        "Following may be raised but are not in function signature: [{}]",
                        raised
                    );
                    return Err(vec![TypeErr::new(&ast.pos, &msg)]);
                }

                if let Some(ret_ty) = ret_ty {
                    ctx.lookup(&ret_ty, &ast.pos)?;
                    let body_ty = TypeName::from(&body_ty.expr_ty(&ast.pos)?);
                    if !ret_ty.is_superset(&body_ty) {
                        let msg = format!("Must have return type {}, was {}", ret_ty, body_ty);
                        return Err(vec![TypeErr::new(&ast.pos, &msg)]);
                    }
                }
            }

            Ok((InferType::default(), env.clone()))
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected variable manipulation")])
    }
}

pub fn arg_types(
    args: &[AST],
    env: &Environment,
    ctx: &Context
) -> TypeResult<HashMap<String, (bool, ExpressionType)>> {
    let mut arg_types = HashMap::new();
    for arg in args {
        // TODO do something with vararg
        let (var, mutable, ty, default) = match &arg.node {
            Node::FunArg { var, mutable, ty, default, .. } => (var, mutable, ty, default),
            _ => return Err(vec![TypeErr::new(&arg.pos, "Expected function argument")])
        };

        let lit = argument_name(var)?;
        let default_ty = match default {
            Some(default) => Some(infer(default, env, ctx)?.0.expr_ty(&var.pos)?),
            None => None
        };

        // TODO if return type is none, then body should not return anything
        // TODO if op overloading, return type Bool or class even if not specified
        if let Some(_type) = ty {
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
        } else if lit == function_arg::concrete::SELF {
            // TODO check that type of self is child of class
            // TODO get actual type of self from Context in case self is child of class
            let (_, class_ty) = env.lookup_indirect("self", &arg.pos)?;
            arg_types.insert(lit.clone(), (*mutable, class_ty));
        } else if let Some(default_ty) = default_ty {
            arg_types.insert(lit.clone(), (*mutable, default_ty));
        } else {
            let msg = format!("Cannot derive type of {}", lit);
            return Err(vec![TypeErr::new(&arg.pos, &msg)]);
        }
    }

    Ok(arg_types)
}
