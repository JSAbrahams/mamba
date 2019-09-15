use crate::parser::ast::{Node, AST};
use crate::type_checker::context::Context;
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::infer_type::InferType;
use crate::type_checker::infer::InferResult;
use crate::type_checker::type_result::TypeErr;

pub fn infer_error(ast: &AST, env: &Environment, ctx: &Context, state: &State) -> InferResult {
    match &ast.node {
        Node::Raise { .. } => Ok((InferType::new(None), env.clone())),

        // TODO verify that errors of raises equal to expr errors
        Node::Raises { .. } => unimplemented!(),
        // TODO traverse arms of handle
        // TODO copy over raises that are not handled in any arms
        Node::Handle { expr_or_stmt, cases } => unimplemented!(),

        Node::Retry =>
            if state.in_handle {
                Ok((InferType::new(None), env.clone()))
            } else {
                Err(vec![TypeErr::new(&ast.pos, "Retry only possible in handle arm")])
            },

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected error")])
    }
}
