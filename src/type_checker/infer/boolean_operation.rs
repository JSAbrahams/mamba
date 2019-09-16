use crate::parser::ast::{Node, AST};
use crate::type_checker::context::Context;
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::TypeErr;

pub fn infer_boolean_op(ast: &AST, env: &Environment, ctx: &Context, state: &State) -> InferResult {
    match &ast.node {
        Node::Is { left, right }
        | Node::IsN { left, right }
        | Node::Neq { left, right }
        | Node::IsA { left, right }
        | Node::IsNA { left, right } => {
            let (left_ty, left_env) = infer(left, env, ctx, state)?;
            let (right_ty, right_env) = infer(right, &left_env, ctx, &state)?;

            if left_ty.expr_type.is_none() {
                Err(vec![TypeErr::new(&left.pos, "Must be expression")])
            } else if right_ty.expr_type.is_none() {
                Err(vec![TypeErr::new(&right.pos, "Must be expression")])
            } else {
                // TODO create type for Boolean
                // TODO add unhandled errors here
                unimplemented!()
            }
        }
        Node::And { left, right } | Node::Or { left, right } => {
            let (left_ty, left_env) = infer(left, env, ctx, state)?;
            let (right_ty, right_env) = infer(right, env, ctx, &state)?;

            match (left_ty.expr_type.clone(), right_ty.expr_type.clone()) {
                (None, _) => Err(vec![TypeErr::new(&left.pos, "Must be expression")]),
                (_, None) => Err(vec![TypeErr::new(&right.pos, "Must be expression")]),
                (Some(left_expr_ty), Some(right_expr_ty)) => {
                    // TODO chain unhandled errors here
                    // TODO check that both are boolean type
                    unimplemented!()
                }
            }
        }
        Node::Not { expr } => {
            let (expr_ty, env) = infer(expr, env, ctx, state)?;
            // TODO check boolean and return
            unimplemented!()
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected boolean operation")])
    }
}
