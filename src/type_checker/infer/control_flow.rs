use crate::parser::ast::{Node, AST};
use crate::type_checker::context::ty::concrete;
use crate::type_checker::context::type_name::concrete::TypeName;
use crate::type_checker::context::Context;
use crate::type_checker::environment::infer_type::InferType;
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::state::StateType::InLoop;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::TypeErr;

pub fn infer_control_flow(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    state: &State
) -> InferResult {
    match &ast.node {
        Node::IfElse { cond, then, _else } => {
            let (cond_type, env) = infer(cond, env, ctx, state)?;
            if cond_type
                != ctx.lookup(&TypeName::new(concrete::BOOL_PRIMITIVE, &vec![]), &ast.pos)?
            {
                return Err(vec![TypeErr::new(&cond.pos, "Expected boolean")]);
            }

            let (then_type, env) = infer(then, &env, ctx, state)?;
            if let Some(_else) = _else {
                let (else_type, else_env) = infer(_else, &env, ctx, state)?;
                Ok((then_type.union(&else_type, &ast.pos)?, env.intersection(else_env)))
            } else {
                Ok((then_type, env))
            }
        }
        Node::While { cond, body } => {
            let (cond_type, cond_env) = infer(cond, env, ctx, state)?;
            if cond_type
                != ctx.lookup(&TypeName::new(concrete::BOOL_PRIMITIVE, &vec![]), &ast.pos)?
            {
                return Err(vec![TypeErr::new(&cond.pos, "Expected boolean")]);
            }

            let (_, env) = infer(body, &cond_env, ctx, &state.clone().is(InLoop)?)?;
            Ok((InferType::new(), env))
        }

        Node::Match { .. } => unimplemented!(),
        Node::Case { .. } => unimplemented!(),

        Node::For { .. } => unimplemented!(),
        Node::In { .. } => unimplemented!(),
        Node::Range { .. } => unimplemented!(),
        Node::Step { .. } => unimplemented!(),

        Node::Break | Node::Continue =>
            if state.in_loop {
                Ok((InferType::new(), env.clone()))
            } else {
                Err(vec![TypeErr::new(&ast.pos, "Cannot occur outside loop")])
            },

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected control flow")])
    }
}
