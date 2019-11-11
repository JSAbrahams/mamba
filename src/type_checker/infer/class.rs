use std::convert::TryFrom;
use std::ops::Deref;

use crate::parser::ast::{Node, AST};
use crate::type_checker::context::type_name::actual::ActualTypeName;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::context::Context;
use crate::type_checker::environment::infer_type::InferType;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::TypeErr;

pub fn infer_class(ast: &AST, env: &Environment, ctx: &Context) -> InferResult {
    match &ast.node {
        Node::Init => Ok((InferType::new(), env.clone())),
        Node::Class { _type, args, parents, body } => {
            // TODO type check arguments and constructor of parent
            for arg in args {
                let id_maybe_type = match &arg.node {
                    Node::VariableDef { id_maybe_type, .. } => id_maybe_type.clone(),
                    Node::FunArg { id_maybe_type, .. } => id_maybe_type.clone(),
                    _ => return Err(vec![TypeErr::new(&arg.pos, "Expected argument")])
                };

                match &id_maybe_type.node {
                    Node::IdType { _type, .. } =>
                        if let Some(_type) = _type {
                            let type_name = TypeName::try_from(_type.deref())?;
                            ctx.lookup(&type_name, &_type.pos)?;
                        },
                    _ => return Err(vec![TypeErr::new(&arg.pos, "Expected identifier")])
                };
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
                let state = env.state.in_class(&ActualTypeName::try_from(_type.deref())?);
                infer(body, &env.new_state(&state), ctx)?;
            }

            Ok((InferType::new(), env.clone()))
        }
        Node::Generic { .. } => Ok((InferType::new(), env.clone())),
        Node::Parent { .. } => Ok((InferType::new(), env.clone())),

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected class or class element")])
    }
}
