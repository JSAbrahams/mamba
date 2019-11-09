use std::convert::TryFrom;
use std::ops::Deref;

use crate::parser::ast::{Node, AST};
use crate::type_checker::context::ty::concrete;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::context::Context;
use crate::type_checker::environment::infer_type::InferType;
use crate::type_checker::environment::name::{match_name, Identifier};
use crate::type_checker::environment::state::StateType::InLoop;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::collection::iterable_generic;
use crate::type_checker::infer::control_flow::match_flow::infer_match;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::TypeErr;

mod match_flow;

pub fn infer_control_flow(ast: &AST, env: &Environment, ctx: &Context) -> InferResult {
    match &ast.node {
        Node::Handle { .. } => infer_match(ast, env, ctx),
        Node::Match { .. } => infer_match(ast, env, ctx),
        Node::Case { .. } => infer_match(ast, env, ctx),

        Node::IfElse { cond, then, _else } => {
            let (cond_type, env) = infer(cond, env, ctx)?;
            let bool_ty = ctx.lookup(&TypeName::from(concrete::BOOL_PRIMITIVE), &ast.pos)?;
            if cond_type.expr_ty(&ast.pos)? != bool_ty {
                let msg = format!("Expected {}, was {}", concrete::BOOL_PRIMITIVE, cond_type);
                return Err(vec![TypeErr::new(&cond.pos, &msg)]);
            }

            let (then_type, then_env) = infer(then, &env, ctx)?;
            if let Some(_else) = _else {
                let (else_type, else_env) = infer(_else, &env, ctx)?;
                Ok((then_type.union(&else_type, &ast.pos)?, then_env.difference(else_env)))
            } else {
                Ok((then_type, then_env))
            }
        }
        Node::While { cond, body } => {
            let (cond_type, env) = infer(cond, env, ctx)?;
            let bool_ty = ctx.lookup(&TypeName::from(concrete::BOOL_PRIMITIVE), &ast.pos)?;
            if cond_type.expr_ty(&cond.pos)? != bool_ty {
                let msg = format!("Expected {}, was {}", concrete::BOOL_PRIMITIVE, cond_type);
                return Err(vec![TypeErr::new(&cond.pos, &msg)]);
            }

            let state = env.state.as_state(InLoop);
            let (body_ty, _) = infer(body, &env.new_state(&state), ctx)?;
            Ok((InferType::new().union_raises(&body_ty.raises), env))
        }

        Node::For { expr, col, body } => {
            let identifier = Identifier::try_from(expr.deref())?;
            let (col_ty, mut env) = infer(col, &env, ctx)?;
            let expr_ty = iterable_generic(&col_ty.expr_ty(&col.pos)?, ctx, &env, &col.pos)?;
            for (id, (mutable, expr_ty)) in match_name(&identifier, &expr_ty, &env, &col.pos)? {
                env.insert(&id, mutable, &expr_ty);
            }

            let state = env.state.as_state(InLoop);
            let (body_ty, _) = infer(body, &env.new_state(&state), ctx)?;
            Ok((InferType::new().add_raises(&body_ty).add_raises(&col_ty), env))
        }

        Node::Range { from, to, step, .. } => {
            let (from_ty, env) = infer(from, env, ctx)?;
            let int_ty = ctx.lookup(&TypeName::from(concrete::INT_PRIMITIVE), &ast.pos)?;
            if from_ty.expr_ty(&from.pos)? != int_ty {
                return Err(vec![TypeErr::new(&from.pos, "Must be integer")]);
            }

            let (to_ty, env) = infer(to, &env, ctx)?;
            if to_ty.expr_ty(&to.pos)? != int_ty {
                return Err(vec![TypeErr::new(&to.pos, "Must be integer")]);
            }

            if let Some(step) = step {
                let (step_ty, env) = infer(step, &env, ctx)?;
                if step_ty.expr_ty(&step.pos)? != int_ty {
                    return Err(vec![TypeErr::new(&step.pos, "Must be integer")]);
                }

                let ty = InferType::from(&ctx.lookup(&TypeName::from(concrete::RANGE), &ast.pos)?);
                Ok((ty.add_raises(&from_ty).add_raises(&to_ty).add_raises(&step_ty), env))
            } else {
                let ty = ctx.lookup(&TypeName::from(concrete::RANGE), &ast.pos)?;
                Ok((InferType::from(&ty).add_raises(&from_ty).add_raises(&to_ty), env))
            }
        }
        Node::Step { amount } => {
            let (ty, env) = infer(amount, env, ctx)?;
            let int_ty = ctx.lookup(&TypeName::from(concrete::INT_PRIMITIVE), &ast.pos)?;
            if ty.expr_ty(&amount.pos)? != int_ty {
                Err(vec![TypeErr::new(&amount.pos, "Must be integer")])
            } else {
                Ok((ty, env))
            }
        }

        Node::Break | Node::Continue =>
            if env.state.in_loop {
                Ok((InferType::new(), env.clone()))
            } else {
                Err(vec![TypeErr::new(&ast.pos, "Cannot occur outside loop")])
            },

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected control flow")])
    }
}
