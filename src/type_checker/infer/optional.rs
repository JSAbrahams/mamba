use crate::parser::ast::{Node, AST};
use crate::type_checker::context::Context;
use crate::type_checker::environment::state::{State, StateType};
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::TypeErr;

pub fn infer_optional(ast: &AST, env: &Environment, ctx: &Context, state: &State) -> InferResult {
    match &ast.node {
        Node::Question { left, right } => {
            let (left_ty, env) = infer(left, env, ctx, &state.as_state(StateType::Nullable))?;
            if left_ty.expr_ty(&ast.pos)?.is_nullable() {
                let (right_ty, env) = infer(right, &env, ctx, state)?;
                Ok((right_ty.union(&left_ty, &ast.pos)?, env))
            } else {
                Err(vec![TypeErr::new(&left.pos, "Type must be nullable")])
            }
        }
        _ => Err(vec![TypeErr::new(&ast.pos, "Expected optional")])
    }
}
