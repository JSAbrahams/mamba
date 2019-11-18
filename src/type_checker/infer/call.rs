use std::collections::HashSet;
use std::convert::TryFrom;
use std::ops::Deref;

use crate::parser::ast::{Node, AST};
use crate::type_checker::context::function_arg;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::context::Context;
use crate::type_checker::environment::expression_type::ExpressionType;
use crate::type_checker::environment::infer_type::InferType;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::TypeErr;

// TODO fix code duplication

pub fn infer_call(ast: &AST, env: &Environment, ctx: &Context) -> InferResult {
    match &ast.node {
        Node::FunctionCall { name, args } => {
            let fun_name = TypeName::try_from(name.deref())?;
            let mut arg_names = vec![];
            let mut raises = HashSet::new();
            let mut env = env.clone();
            for arg in args {
                let (ty, new_env) = infer(arg, &env, ctx)?;
                arg_names.push(TypeName::from(&ty.expr_ty(&arg.pos)?));
                env = new_env;
                raises = raises.union(&ty.raises).cloned().collect();
            }

            // TODO add functions to env that may not have return type
            match env.lookup(&fun_name.clone().single(&name.pos)?.name(&name.pos)?, &name.pos) {
                Ok(expr_ty) => Ok((
                    InferType::from(&expr_ty.anon_fun(&arg_names, &ast.pos)?).union_raises(&raises),
                    env.clone()
                )),
                Err(_) => match ctx.lookup_fun(&fun_name, &arg_names, &ast.pos) {
                    // else, see if constructor of type
                    Err(_) => {
                        let expr_ty = ctx.lookup(&fun_name, &name.pos)?;
                        expr_ty.constructor(&arg_names, &ast.pos)?;
                        Ok((InferType::from(&expr_ty).union_raises(&raises), env))
                    }
                    Ok(ok) => Ok((ok.union_raises(&raises), env))
                }
            }
        }

        Node::PropertyCall { instance, property } => {
            let (instance_ty, env) = infer(instance, env, ctx)?;
            let expr_ty = instance_ty.expr_ty(&instance.pos)?;

            let mutable = match &instance.node {
                Node::VariableDef { id_maybe_type, .. } => match &id_maybe_type.node {
                    Node::IdType { mutable, .. } => mutable.clone(),
                    _ => false
                },
                Node::IdType { id, .. } => match &id.node {
                    Node::Id { lit } => env.lookup_indirect(lit, &id.pos)?.0,
                    _ => false
                },
                Node::Id { lit } => env.lookup_indirect(&lit, &instance.pos)?.0,
                Node::_Self => env.lookup_indirect(function_arg::concrete::SELF, &instance.pos)?.0,
                _ => false
            };

            let (ty, env) = property_call(mutable, &expr_ty, property, ctx, &env)?;
            Ok((ty.union_raises(&instance_ty.raises), env))
        }
        _ => Err(vec![TypeErr::new(&ast.pos, "Expected call")])
    }
}

fn property_call(
    mutable: bool,
    instance: &ExpressionType,
    property: &AST,
    ctx: &Context,
    env: &Environment
) -> InferResult {
    match &property.node {
        Node::PropertyCall { instance: inner_instance, property } => {
            let (instance, property_mutable, raises) = match &inner_instance.node {
                Node::Id { lit } => {
                    let field = instance.field(&lit, &property.pos)?;
                    let msg = format!("Cannot get type of field {}", field);
                    let field_ty_name = &field.ty.ok_or(vec![TypeErr::new(&property.pos, &msg)])?;
                    (ctx.lookup(&field_ty_name, &property.pos)?, field.mutable, HashSet::new())
                }
                Node::FunctionCall { name, args } => {
                    let name = match &name.node {
                        Node::Id { lit } => lit.clone(),
                        _ => return Err(vec![TypeErr::new(&name.pos, "Expected identifier")])
                    };

                    let mut raises = HashSet::new();
                    let mut arg_names = vec![];
                    let mut env = env.clone();
                    for arg in args {
                        let (arg_ty, new_env) = infer(arg, &env, ctx)?;
                        arg_names.push(TypeName::from(&arg_ty.expr_ty(&arg.pos)?));
                        raises = raises.union(&arg_ty.raises).cloned().collect();
                        env = new_env;
                    }

                    let function = instance.fun(&name, &arg_names, &property.pos)?;
                    let function_self_mut = function.self_mutable == Some(false);
                    if !function_self_mut && mutable {
                        let msg = format!("Cannot call {}, which expects self to be mutable", name);
                        return Err(vec![TypeErr::new(&property.pos, &msg)]);
                    }

                    let function_ty_name = &function
                        .ty()
                        .ok_or(vec![TypeErr::new(&property.pos, "Cannot get type of function")])?;
                    (
                        ctx.lookup(&function_ty_name, &property.pos)?,
                        function_self_mut,
                        function.raises
                    )
                }
                _ =>
                    return Err(vec![TypeErr::new(
                        &inner_instance.pos,
                        "Expected function or field"
                    )]),
            };

            let (infer_ty, env) =
                property_call(mutable && property_mutable, &instance, property, ctx, &env)?;
            Ok((infer_ty.union_raises(&raises), env))
        }

        _ => final_property_call(mutable, instance, property, ctx, env)
    }
}

fn final_property_call(
    mutable: bool,
    instance: &ExpressionType,
    property: &AST,
    ctx: &Context,
    env: &Environment
) -> InferResult {
    match &property.node {
        Node::Id { lit } => {
            let field = instance.field(&lit, &property.pos)?;
            let msg = format!("Cannot get type of field {}", field);
            let field_ty_name = &field.ty.ok_or(vec![TypeErr::new(&property.pos, &msg)])?;
            Ok((InferType::from(&ctx.lookup(&field_ty_name, &property.pos)?), env.clone()))
        }

        Node::Reassign { left, right } => {
            let (id, ty) = match &left.node {
                Node::Id { lit } => (lit.clone(), None),
                Node::IdType { id, _type, .. } => match (&id.node, &_type) {
                    (Node::Id { lit }, Some(ty)) =>
                        (lit.clone(), Some((TypeName::try_from(ty.deref())?, ty.pos.clone()))),
                    (Node::Id { lit }, None) => (lit.clone(), None),
                    _ => return Err(vec![TypeErr::new(&left.pos, "Expected identifier")])
                },
                _ => return Err(vec![TypeErr::new(&left.pos, "Expected identifier")])
            };

            let field = instance.field(&id, &left.pos)?;
            if let Some((ty, pos)) = ty {
                if field.ty()? != ty {
                    let msg = format!("Expected {}, given {}", field.ty()?, ty);
                    return Err(vec![TypeErr::new(&pos, &msg)]);
                }
            }

            if !mutable || !field.mutable {
                let msg = format!("Cannot re-assign to immutable {}", id);
                return Err(vec![TypeErr::new(&left.pos, &msg)]);
            }

            let (right_ty, env) = infer(right, &env, ctx)?;
            let right_name = TypeName::from(&right_ty.expr_ty(&right.pos)?);
            if field.ty()? != right_name {
                let msg = format!("Expected {}, was {}", field.ty()?, right_name);
                return Err(vec![TypeErr::new(&right.pos, &msg)]);
            }

            let infer_type = InferType::new();
            Ok((infer_type.union_raises(&right_ty.raises).union_raises(&right_ty.raises), env))
        }

        Node::FunctionCall { name, args } => {
            let name = match &name.node {
                Node::Id { lit } => lit.clone(),
                _ => return Err(vec![TypeErr::new(&name.pos, "Expected identifier")])
            };

            let mut raises = HashSet::new();
            let mut arg_names = vec![];
            let mut env = env.clone();
            for arg in args {
                let (arg_ty, new_env) = infer(arg, &env, ctx)?;
                arg_names.push(TypeName::from(&arg_ty.expr_ty(&arg.pos)?));
                raises = raises.union(&arg_ty.raises).cloned().collect();
                env = new_env;
            }

            let function = instance.fun(&name, &arg_names, &property.pos)?;
            if function.self_mutable == Some(true) && !mutable {
                let msg = format!("Cannot call {}, which expects self to be mutable", name);
                return Err(vec![TypeErr::new(&property.pos, &msg)]);
            }

            let function_ty_name = &function
                .ty()
                .ok_or(vec![TypeErr::new(&property.pos, "Cannot get type of function")])?;
            let infer_type = InferType::from(&ctx.lookup(&function_ty_name, &property.pos)?);
            Ok((infer_type.union_raises(&raises), env))
        }

        _ => return Err(vec![TypeErr::new(&property.pos, "Expected property or function")])
    }
}
