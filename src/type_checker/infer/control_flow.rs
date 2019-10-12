use crate::parser::ast::{Node, AST};
use crate::type_checker::context::ty::concrete;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::context::Context;
use crate::type_checker::environment::identifier::{match_name, Identifier};
use crate::type_checker::environment::infer_type::InferType;
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::state::StateType::InLoop;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::collection::iterable_generic;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::TypeErr;
use std::convert::TryFrom;
use std::ops::Deref;

pub fn infer_control_flow(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    state: &State
) -> InferResult {
    match &ast.node {
        Node::IfElse { cond, then, _else } => {
            let (cond_type, env) = infer(cond, env, ctx, state)?;
            if cond_type != ctx.lookup(&TypeName::from(concrete::BOOL_PRIMITIVE), &ast.pos)? {
                return Err(vec![TypeErr::new(
                    &cond.pos,
                    &format!("Expected {}, was {}", concrete::BOOL_PRIMITIVE, cond_type)
                )]);
            }

            let (then_type, then_env) = infer(then, &env, ctx, state)?;
            if let Some(_else) = _else {
                let (else_type, else_env) = infer(_else, &env, ctx, state)?;
                Ok((then_type.union(&else_type, &ast.pos)?, then_env.difference(else_env)))
            } else {
                Ok((then_type, then_env))
            }
        }
        Node::While { cond, body } => {
            let (cond_type, cond_env) = infer(cond, env, ctx, state)?;
            if cond_type != ctx.lookup(&TypeName::from(concrete::BOOL_PRIMITIVE), &ast.pos)? {
                return Err(vec![TypeErr::new(
                    &cond.pos,
                    &format!("Expected {}, was {}", concrete::BOOL_PRIMITIVE, cond_type)
                )]);
            }

            let (_, env) = infer(body, &cond_env, ctx, &state.clone().is(InLoop)?)?;
            Ok((InferType::new(), env))
        }

        Node::Match { .. } => unimplemented!(),
        Node::Case { .. } => unimplemented!(),

        Node::For { expr, col, body } => {
            let identifier = Identifier::try_from(expr.deref())?;
            let (col_ty, mut env) = infer(col, &env, ctx, state)?;
            let expr_ty = iterable_generic(&col_ty.expr_ty(&col.pos)?, ctx, &col.pos)?;
            for (mutable, id, expr_ty) in match_name(&identifier, &expr_ty, &col.pos)? {
                env = env.insert(&id, mutable, &expr_ty)?;
            }

            let (body_ty, env) = infer(body, &env, ctx, state)?;
            Ok((InferType::new().add_raises(&body_ty.raises).add_raises(&col_ty.raises), env))
        }

        Node::Range { from, to, step, .. } => {
            let (from_ty, env) = infer(from, env, ctx, state)?;
            if from_ty != ctx.lookup(&TypeName::from(concrete::INT_PRIMITIVE), &ast.pos)? {
                return Err(vec![TypeErr::new(&from.pos, "Must be integer")]);
            }

            let (to_ty, env) = infer(to, &env, ctx, state)?;
            if to_ty != ctx.lookup(&TypeName::from(concrete::INT_PRIMITIVE), &ast.pos)? {
                return Err(vec![TypeErr::new(&to.pos, "Must be integer")]);
            }

            if let Some(step) = step {
                let (step_ty, env) = infer(step, &env, ctx, state)?;
                if step_ty
                    != ctx.lookup(&TypeName::new(concrete::INT_PRIMITIVE, &vec![]), &ast.pos)?
                {
                    return Err(vec![TypeErr::new(&step.pos, "Must be integer")]);
                }

                let ty = ctx
                    .lookup(&TypeName::new(concrete::RANGE, &vec![]), &ast.pos)?
                    .add_raises(&from_ty.raises)
                    .add_raises(&to_ty.raises)
                    .add_raises(&step_ty.raises);
                Ok((ty, env))
            } else {
                let ty = ctx.lookup(&TypeName::new(concrete::RANGE, &vec![]), &ast.pos)?;
                Ok((ty.add_raises(&from_ty.raises).add_raises(&to_ty.raises), env))
            }
        }
        Node::Step { amount } => {
            let (ty, env) = infer(amount, env, ctx, state)?;
            if ty != ctx.lookup(&TypeName::from(concrete::INT_PRIMITIVE), &ast.pos)? {
                Err(vec![TypeErr::new(&amount.pos, "Must be integer")])
            } else {
                Ok((ty, env))
            }
        }

        Node::Break | Node::Continue =>
            if state.in_loop {
                Ok((InferType::new(), env.clone()))
            } else {
                Err(vec![TypeErr::new(&ast.pos, "Cannot occur outside loop")])
            },

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected control flow")])
    }
}
