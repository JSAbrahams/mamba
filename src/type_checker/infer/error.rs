use crate::parser::ast::{Node, AST};
use crate::type_checker::context::Context;
use crate::type_checker::environment::infer_type::InferType;
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::TypeErr;

pub fn infer_error(ast: &AST, env: &Environment, ctx: &Context, state: &State) -> InferResult {
    match &ast.node {
        Node::Raise { error } => {
            let (ty, _) = infer(error, env, ctx, state)?;
            Ok((
                InferType::new().add_raise_from_type(&ty, &ast.pos)?.add_raises(&ty.raises),
                env.clone()
            ))
        }

        // TODO verify that errors of raises equal to expr errors
        Node::Raises { expr_or_stmt, errors } => unimplemented!(),
        Node::Handle { expr_or_stmt, cases } => unimplemented!(),

        Node::Retry =>
            if !(state.in_handle) {
                Err(vec![TypeErr::new(&ast.pos, "Retry only possible in handle arm")])
            } else {
                Ok((InferType::new(), env.clone()))
            },

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected error")])
    }
}
