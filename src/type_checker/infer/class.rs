use std::convert::TryFrom;
use std::ops::Deref;

use crate::parser::ast::{Node, AST};
use crate::type_checker::context::type_name::actual::ActualTypeName;
use crate::type_checker::context::Context;
use crate::type_checker::environment::infer_type::InferType;
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::TypeErr;

pub fn infer_class(ast: &AST, env: &Environment, ctx: &Context, state: &State) -> InferResult {
    match &ast.node {
        Node::Init => Ok((InferType::new(), env.clone())),
        Node::Class { _type, body, .. } => {
            if let Some(body) = body {
                let state = state.in_class(&ActualTypeName::try_from(_type.deref())?);
                infer(body, env, ctx, &state)?;
            }

            Ok((InferType::new(), env.clone()))
        }
        Node::Generic { .. } => Ok((InferType::new(), env.clone())),
        Node::Parent { .. } => Ok((InferType::new(), env.clone())),

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected class or class element")])
    }
}
