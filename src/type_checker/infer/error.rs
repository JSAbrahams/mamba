use std::convert::TryFrom;

use crate::parser::ast::{Node, AST};
use crate::type_checker::context::type_name::actual::ActualTypeName;
use crate::type_checker::context::Context;
use crate::type_checker::environment::infer_type::InferType;
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::TypeErr;
use std::collections::HashSet;
use std::iter::FromIterator;

pub fn infer_error(ast: &AST, env: &Environment, ctx: &Context, state: &State) -> InferResult {
    match &ast.node {
        Node::Raise { error } => {
            let (ty, env) = infer(error, env, ctx, state)?;
            let actual_ty = ty.expr_ty(&error.pos)?.single(&error.pos)?.actual_ty();
            let set = HashSet::from_iter(vec![ActualTypeName::from(&actual_ty)].into_iter());
            Ok((InferType::new().union_raises(&set).add_raises(&ty), env))
        }
        Node::Raises { expr_or_stmt, errors } => {
            let (ty, env) = infer(expr_or_stmt, env, ctx, state)?;
            let errors = errors.iter().map(ActualTypeName::try_from).collect::<Result<_, _>>()?;
            if ty.raises.is_superset(&errors) {
                Ok((ty, env))
            } else {
                Err(vec![TypeErr::new(&expr_or_stmt.pos, "Errors don't match expr or statement")])
            }
        }

        Node::Handle { .. } => unimplemented!(),

        Node::Retry =>
            if !(state.in_handle) {
                Err(vec![TypeErr::new(&ast.pos, "Retry only possible in handle arm")])
            } else {
                Ok((InferType::new(), env.clone()))
            },

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected error")])
    }
}
