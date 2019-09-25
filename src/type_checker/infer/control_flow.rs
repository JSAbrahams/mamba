use crate::parser::ast::{Node, AST};
use crate::type_checker::context::generic::type_name::GenericTypeName;
use crate::type_checker::context::{concrete, Context};
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::state::StateType::InLoop;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::infer_type::InferType;
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
            let (cond_type, cond_env) = infer(cond, env, ctx, state)?;
            let cond_expr_ty = cond_type.expr_tys(&ast.pos)?;
            let cond_actual_ty = cond_expr_ty.get_actual_ty(&ast.pos)?;
            if cond_actual_ty.ty(&ast.pos)?
                != ctx.lookup(&GenericTypeName::new(concrete::BOOL_PRIMITIVE), &ast.pos)?
            {
                return Err(vec![TypeErr::new(&cond.pos, "Expected boolean")]);
            }

            let (then_type, then_env) = infer(then, &cond_env, ctx, state)?;

            if let Some(_else) = _else {
                let (else_type, else_env) = infer(_else, &cond_env, ctx, state)?;
                Ok((then_type.union(&else_type, &ast.pos)?, then_env.intersection(else_env)))
            } else {
                Ok((then_type, then_env))
            }
        }
        Node::While { cond, body } => {
            let (cond_type, cond_env) = infer(cond, env, ctx, state)?;
            let cond_expr_ty = cond_type.expr_tys(&ast.pos)?;
            let cond_actual_ty = cond_expr_ty.get_actual_ty(&ast.pos)?;
            let cond_ty = cond_actual_ty.ty(&ast.pos)?;
            if cond_ty != ctx.lookup(&GenericTypeName::new(concrete::BOOL_PRIMITIVE), &ast.pos)? {
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
