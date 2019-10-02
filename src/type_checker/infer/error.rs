use crate::parser::ast::{Node, AST};
use crate::type_checker::context::type_name::concrete::TypeName;
use crate::type_checker::context::type_name::generic::GenericTypeName;
use crate::type_checker::context::Context;
use crate::type_checker::environment::infer_type::InferType;
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::TypeErr;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::iter::FromIterator;

pub fn infer_error(ast: &AST, env: &Environment, ctx: &Context, state: &State) -> InferResult {
    match &ast.node {
        Node::Raise { error } => {
            let error_name = TypeName::try_from(error)?;
            let err = ctx.lookup(error_name, &error.pos)?;
            Ok((InferType::new().raises(HashSet::from_iter(vec![err].iter())), env.clone()))
        }

        // TODO verify that errors of raises equal to expr errors
        Node::Raises { expr_or_stmt, errors } => {
            let (ty, env) = infer(expr_or_stmt, env, ctx, state)?;
            let errs = errors.iter().map(|e| (e.pos, GenericTypeName::try_from(e))).collect()?;
            let errs = errs.iter().map(|(pos, e)| ctx.lookup(e, pos)).collect()?;

            let unhandled_errs = ty.raises.difference(&errs).collect();
            let redundant_raises = errs.difference(&ty.raises).collect();

            if !unhandled_errs.is_empty() {
                Err(vec![TypeErr::new(
                    &ast.pos,
                    format!("Errors not mentioned: {:#?}", unhandled_errs).as_ref()
                )])
            } else if !redundant_raises.is_empty() {
                Err(vec![TypeErr::new(
                    &ast.pos,
                    format!("Unexpected Raises: {:#?}", redundant_raises).as_ref()
                )])
            } else {
                Ok((ty, env))
            }
        }
        Node::Handle { expr_or_stmt, cases } => unimplemented!(),

        Node::Retry =>
            if state.in_handle {
                Ok((InferType::new(), env.clone()))
            } else {
                Err(vec![TypeErr::new(&ast.pos, "Retry only possible in handle arm")])
            },

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected error")])
    }
}
