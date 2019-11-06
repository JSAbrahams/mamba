use crate::parser::ast::{Node, AST};
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::context::Context;
use crate::type_checker::environment::infer_type::InferType;
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::TypeErr;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::ops::Deref;

pub fn infer_call(ast: &AST, env: &Environment, ctx: &Context, state: &State) -> InferResult {
    match &ast.node {
        Node::FunctionCall { name, args } => {
            let fun_name = TypeName::try_from(name.deref())?;
            let mut arg_names = vec![];
            let mut raises = HashSet::new();
            let mut env = env.clone();
            for arg in args {
                let (ty, new_env) = infer(arg, &env, ctx, state)?;
                arg_names.push(TypeName::from(&ty.expr_ty(&arg.pos)?));
                env = new_env;
                raises = raises.union(&ty.raises).cloned().collect();
            }

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
                    Ok(ok) => Ok((ok, env))
                }
            }
        }

        Node::PropertyCall { instance, property } => {
            let (instance_ty, env) = infer(instance, env, ctx, state)?;
            let expr_ty = instance_ty.expr_ty(&instance.pos)?;
            let property_ty = match &property.node {
                Node::Id { lit } => {
                    let field = expr_ty.field(&lit, state.nullable, &property.pos)?;
                    let field_ty_name = &field
                        .ty
                        .ok_or(vec![TypeErr::new(&property.pos, "Cannot get type of field")])?;
                    ctx.lookup(&field_ty_name, &property.pos)
                }
                Node::FunctionCall { name, args } => {
                    let name = match &name.node {
                        Node::Id { lit } => lit.clone(),
                        _ => return Err(vec![TypeErr::new(&name.pos, "Expected identifier")])
                    };

                    let mut arg_names = vec![];
                    let mut env = env.clone();
                    for arg in args {
                        let (arg_ty, new_env) = infer(arg, &env, ctx, state)?;
                        arg_names.push(TypeName::from(&arg_ty.expr_ty(&arg.pos)?));
                        env = new_env;
                    }

                    let function = expr_ty.fun(&name, &arg_names, state.nullable, &ast.pos)?;
                    let function_ty_name = &function
                        .ty()
                        .ok_or(vec![TypeErr::new(&property.pos, "Cannot get type of function")])?;
                    ctx.lookup(&function_ty_name, &property.pos)
                }
                _ => return Err(vec![TypeErr::new(&property.pos, "Expected property or function")])
            }?;

            Ok((InferType::from(&property_ty).union_raises(&instance_ty.raises), env))
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected class or class element")])
    }
}
