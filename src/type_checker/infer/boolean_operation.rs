use crate::parser::ast::{Node, AST};
use crate::type_checker::context::Context;
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::TypeErr;

pub fn infer_boolean_op(ast: &AST, env: &Environment, ctx: &Context, state: &State) -> InferResult {
    match &ast.node {
        Node::Is { left, right } => instance_cmp(left, right, env, ctx, state),
        Node::IsN { left, right } => instance_cmp(left, right, env, ctx, state),
        Node::Neq { left, right } => instance_cmp(left, right, env, ctx, state),
        Node::IsA { left, right } => instance_cmp(left, right, env, ctx, state),
        Node::IsNA { left, right } => instance_cmp(left, right, env, ctx, state),
        Node::Not { expr } => unimplemented!(),
        Node::And { left, right } | Node::Or { left, right } => {
            let (left_ty, left_env) = infer(left, env, ctx, state)?;
            let (right_ty, right_env) = infer(right, env, ctx, &state)?;

            match (left_ty.expr_type.clone(), right_ty.expr_type.clone()) {
                (None, _) => Err(vec![TypeErr::new(&left.pos, "Must be expression")]),
                (_, None) => Err(vec![TypeErr::new(&right.pos, "Must be expression")]),
                (Some(left_expr_ty), Some(right_expr_ty)) =>
                    Ok((left_ty.raises(right_ty.raises), right_env)),
            }
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected boolean operation")])
    }
}

fn instance_cmp(
    left: &AST,
    right: &AST,
    env: &Environment,
    ctx: &Context,
    state: &State
) -> InferResult {
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
