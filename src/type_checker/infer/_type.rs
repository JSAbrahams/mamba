use std::convert::TryFrom;
use std::ops::Deref;

use crate::parser::ast::{Node, AST};
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::context::Context;
use crate::type_checker::environment::infer_type::InferType;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::TypeErr;

pub fn infer_type(ast: &AST, env: &Environment, ctx: &Context) -> InferResult {
    match &ast.node {
        Node::TypeDef { isa, body, _type, .. } => {
            if let Some(isa) = isa {
                infer(isa, env, ctx)?;
            }
            if let Some(body) = body {
                let class = TypeName::try_from(_type.deref())?;
                let env = env.in_class(false, &ctx.lookup(&class, &_type.pos)?);
                infer(body, &env, ctx)?;
            }

            Ok((InferType::default(), env.clone()))
        }
        Node::TypeAlias { isa, conditions, _type } => {
            infer(isa, env, ctx)?;
            for condition in conditions {
                let class = TypeName::try_from(isa.deref())?;
                let env = env.in_class(false, &ctx.lookup(&class, &_type.pos)?);
                infer(condition, &env, ctx)?;
            }
            Ok((InferType::default(), env.clone()))
        }
        Node::TypeTup { types } => {
            for ty in types {
                infer(ty, env, ctx)?;
            }
            Ok((InferType::default(), env.clone()))
        }
        Node::TypeUnion { types } => {
            for ty in types {
                infer(ty, env, ctx)?;
            }
            Ok((InferType::default(), env.clone()))
        }
        Node::Type { id, generics } => {
            let id = match &id.node {
                Node::Id { lit } => lit.clone(),
                _ => return Err(vec![TypeErr::new(&id.pos, "Expected identifier")])
            };

            // TODO do something with generics
            ctx.lookup(&TypeName::from(id.as_str()), &ast.pos)?;
            for generic in generics {
                infer(generic, env, ctx)?;
            }
            Ok((InferType::default(), env.clone()))
        }
        Node::TypeFun { args, ret_ty } => {
            for arg in args {
                infer(arg, env, ctx)?;
            }
            infer(ret_ty, env, ctx)?;
            Ok((InferType::default(), env.clone()))
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected type")])
    }
}
