use std::collections::HashSet;
use std::ops::Deref;

use crate::parser::ast::{Node, AST};
use crate::type_checker::context::function::generic::function_name;
use crate::type_checker::context::function_arg;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::infer_type::expression::ExpressionType;
use crate::type_checker::infer_type::InferType;
use crate::type_checker::type_name::TypeName;
use crate::type_checker::type_result::TypeErr;
use std::convert::TryFrom;

pub fn infer_call(ast: &AST, env: &Environment, ctx: &Context) -> InferResult {
    match &ast.node {
        // TODO split up application logic
        Node::ConstructorCall { name, args } => {
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

            let expr_ty = ctx.lookup(&fun_name, &name.pos)?;
            expr_ty.constructor(&arg_names, &ast.pos)?;
            Ok((InferType::from(&expr_ty).union_raises(&raises), env))
        }
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
                    env
                )),
                Err(_) => match ctx.lookup_fun(&fun_name, &arg_names, &ast.pos) {
                    Err(err) => Err(err),
                    Ok(ok) => Ok((ok.union_raises(&raises), env))
                }
            }
        }

        Node::PropertyCall { instance, property } => {
            let (instance_ty, env) = infer(instance, env, ctx)?;
            let expr_ty = instance_ty.expr_ty(&instance.pos)?;
            let mutable = match &instance.node {
                Node::VariableDef { mutable, .. } => *mutable,
                Node::ExpressionType { expr, .. } => match &expr.node {
                    Node::Id { lit } => env.lookup_indirect(lit, &expr.pos)?.0,
                    _ => false
                },
                Node::Id { lit } => env.lookup_indirect(&lit, &instance.pos)?.0,
                Node::_Self => env.lookup_indirect(function_arg::concrete::SELF, &instance.pos)?.0,
                _ => false
            };

            let ((_, ty), env) = property_call(mutable, &expr_ty, property, ctx, &env)?;
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
) -> InferResult<(bool, InferType)> {
    match &property.node {
        Node::PropertyCall { instance: inner_instance, property } => {
            let ((mutable, inner_ty), env) =
                property_call(mutable, instance, inner_instance, ctx, env)?;
            let inner_expr = inner_ty.expr_ty(&inner_instance.pos)?;
            property_call(mutable, &inner_expr, property, ctx, &env)
        }

        Node::Id { lit } => {
            let field = instance.field(&lit, &property.pos)?;
            if field.private && field.in_class != env.state.in_class {
                return Err(vec![TypeErr::new(&property.pos, &format!("{} is private", lit))]);
            }

            let msg = format!("Cannot get type of field {}", field);
            let field_ty_name = &field.ty.ok_or_else(|| vec![TypeErr::new(&property.pos, &msg)])?;
            let field_ty = InferType::from(&ctx.lookup(&field_ty_name, &property.pos)?);
            Ok(((field.mutable && mutable, field_ty), env.clone()))
        }

        Node::Reassign { left, right } => {
            let (id, ty) = match &left.node {
                Node::Id { lit } => (lit.clone(), None),
                Node::ExpressionType { expr, ty, .. } => match (&expr.node, &ty) {
                    (Node::Id { lit }, Some(ty)) =>
                        (lit.clone(), Some((TypeName::try_from(ty.deref())?, ty.pos.clone()))),
                    (Node::Id { lit }, None) => (lit.clone(), None),
                    _ => return Err(vec![TypeErr::new(&left.pos, "Expected identifier")])
                },
                Node::FunctionCall { .. } =>
                    return Err(vec![TypeErr::new(&left.pos, "Cannot assign to function call")]),
                _ => return Err(vec![TypeErr::new(&left.pos, "Expected identifier")])
            };

            let field = instance.field(&id, &left.pos)?;
            if field.private && field.in_class != env.state.in_class {
                return Err(vec![TypeErr::new(&property.pos, &format!("{} is private", id))]);
            } else if !mutable || !field.mutable {
                let msg = format!("Cannot re-assign to immutable {}", id);
                return Err(vec![TypeErr::new(&left.pos, &msg)]);
            } else if let Some((ty, pos)) = ty {
                if field.ty()? != ty {
                    let msg = format!("Expected {}, given {}", field.ty()?, ty);
                    return Err(vec![TypeErr::new(&pos, &msg)]);
                }
            }

            let (right_ty, env) = infer(right, &env, ctx)?;
            let right_name = TypeName::from(&right_ty.expr_ty(&right.pos)?);
            if field.ty()? != right_name {
                let msg = format!("Expected {}, was {}", field.ty()?, right_name);
                return Err(vec![TypeErr::new(&right.pos, &msg)]);
            }

            let infer_type =
                InferType::default().union_raises(&right_ty.raises).union_raises(&right_ty.raises);
            Ok(((false, infer_type), env))
        }

        Node::FunctionCall { name, args } => {
            let name = function_name(name)?;
            // TODO remove this once fun takes ActualTypeName
            let name = name.as_single(&property.pos)?.0;

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
            if function.private && function.in_class != env.state.in_class {
                return Err(vec![TypeErr::new(&property.pos, &format!("{} is private", name))]);
            } else if function.self_mutable == Some(true) && !mutable {
                let msg = format!("Cannot call {}, which expects self to be mutable", name);
                return Err(vec![TypeErr::new(&property.pos, &msg)]);
            }

            let function_ty_name = &function
                .ty()
                .ok_or_else(|| vec![TypeErr::new(&property.pos, "Cannot get type of function")])?;
            let infer_type = InferType::from(&ctx.lookup(&function_ty_name, &property.pos)?);
            Ok(((true, infer_type.union_raises(&raises)), env))
        }

        _ => Err(vec![TypeErr::new(&property.pos, "Expected property or function")])
    }
}
