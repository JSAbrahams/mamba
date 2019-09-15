use crate::parser::ast::{Node, AST};
use crate::type_checker::context::Context;
use crate::type_checker::environment::expression_type::ExpressionType;
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::state::StateType::InLoop;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::TypeErr;

pub fn infer_collection(
    ast: &Box<AST>,
    env: &Environment,
    ctx: &Context,
    state: &State
) -> InferResult {
    match &ast.node {
        Node::IfElse { cond, then, _else } => {
            let (cond_type, cond_env) = infer(cond, env, ctx, state)?;
            let (then_type, then_env) = infer(then, &cond_env, ctx, state)?;

            if let Some(_else) = _else {
                let (else_type, else_env) = infer(_else, &cond_env, ctx, state)?;
                Ok((then_type, then_env.intersection(else_env), state.clone()))
            } else {
                Ok((then_type, then_env, state.clone()))
            }
        }
        Node::Match { .. } => unimplemented!(),
        Node::Case { .. } => unimplemented!(),
        Node::For { .. } => unimplemented!(),
        Node::In { .. } => unimplemented!(),
        Node::Range { .. } => unimplemented!(),
        Node::Step { .. } => unimplemented!(),
        Node::While { cond, body } => {
            let (cond_type, cond_env) = infer(cond, env, ctx, state)?;
            let (_, body_env) = infer(body, &cond_env, ctx, &state.is(InLoop)?)?;
            Ok((None, env.intersection(body_env), state.clone()))
        }
        Node::Break =>
            if state.in_loop {
                Ok((None, env.clone(), state.clone()))
            } else {
                Err(vec![TypeErr::new(&ast.pos, "Cannot occur outside loop")])
            },
        Node::Continue => Ok((None, env.clone(), state.clone())),

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected control flow")])
    }
}
