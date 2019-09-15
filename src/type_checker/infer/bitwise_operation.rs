use crate::parser::ast::{Node, AST};
use crate::type_checker::context::Context;
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::TypeErr;

pub fn infer_bitwise_op(ast: &AST, env: &Environment, ctx: &Context, state: &State) -> InferResult {
    match &ast.node {
        Node::BAnd { left, right }
        | Node::BOr { left, right }
        | Node::BXOr { left, right }
        | Node::BLShift { left, right }
        | Node::BRShift { left, right } => {
            let (left_ty, left_env) = infer(left, env, ctx, state)?;
            let (right_ty, right_env) = infer(right, env, ctx, &state)?;

            // TODO create type for integers
            // TODO chain raised errors of above types
            unimplemented!()
        }
        Node::BOneCmpl { expr } => {
            let (expr_ty, env) = infer(expr, env, ctx, state)?;
            // TODO chain raised errors of above types
            unimplemented!()
        }
        _ => Err(vec![TypeErr::new(&ast.pos, "Expected bitwise operation")])
    }
}
