use crate::parser::ast::{Node, AST};
use crate::type_checker::context::Context;
use crate::type_checker::environment::expression_type::ExpressionType;
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::TypeErr;

pub fn infer_error(ast: &Box<AST>, env: &Environment, ctx: &Context, state: &State) -> InferResult {
    match &ast.node {
        Node::Raises { .. } => unimplemented!(),
        Node::Raise { .. } => unimplemented!(),
        Node::Handle { expr_or_stmt, cases } =>
            if let (Some(expr_type), expr_env) = infer(expr_or_stmt, env, ctx, state)? {
                let state = state.clone().unhandled(&expr_type.raises);
                unimplemented!()
            } else {
                Err(vec![TypeErr::new(&ast.pos, "Expected expression")])
            },
        Node::Retry => unimplemented!(),

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected error")])
    }
}
