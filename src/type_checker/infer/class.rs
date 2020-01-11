use std::convert::TryFrom;
use std::ops::Deref;

use crate::parser::ast::{Node, AST};
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::infer_type::InferType;
use crate::type_checker::type_name::TypeName;
use crate::type_checker::type_result::TypeErr;

pub fn infer_class(ast: &AST, env: &Environment, ctx: &Context) -> InferResult {
    match &ast.node {
        Node::Init => Ok((InferType::default(), env.clone())),
        Node::Class { ty, args, parents, body, .. } => {
            // TODO type check arguments and constructor of parent
            for arg in args {
                let ty = match &arg.node {
                    Node::VariableDef { ty, .. } => ty.clone(),
                    Node::FunArg { ty, .. } => ty.clone(),
                    _ => return Err(vec![TypeErr::new(&arg.pos, "Expected argument")])
                };

                if let Some(ty) = ty {
                    let type_name = TypeName::try_from(ty.deref())?;
                    ctx.lookup(&type_name, &ty.pos)?;
                }
            }

            for parent in parents {
                // TODO check if name of parent is valid at earlier stage
                // TODO check generics
                // TODO check if parent constructor has correct arguments
                match &parent.node {
                    Node::Parent { id, .. } => {
                        let type_name = TypeName::try_from(id.deref())?;
                        ctx.lookup(&type_name, &id.pos)?;
                    }
                    _ => return Err(vec![TypeErr::new(&parent.pos, "Expected name")])
                }
            }

            if let Some(body) = body {
                let class = TypeName::try_from(ty.deref())?;
                let env = env.in_class(false, &ctx.lookup(&class, &ty.pos)?);
                infer(body, &env, ctx)?;
            }

            Ok((InferType::default(), env.clone()))
        }
        Node::Generic { .. } => Ok((InferType::default(), env.clone())),
        Node::Parent { .. } => Ok((InferType::default(), env.clone())),

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected class or class element")])
    }
}
