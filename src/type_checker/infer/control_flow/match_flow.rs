use std::convert::TryFrom;
use std::ops::Deref;

use crate::parser::ast::{Node, AST};
use crate::type_checker::context::Context;
use crate::type_checker::environment::infer_type::InferType;
use crate::type_checker::environment::name::{match_name, Identifier};
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::TypeErr;

// TODO add pattern matching type checking

pub fn infer_match(ast: &AST, env: &Environment, ctx: &Context, state: &State) -> InferResult {
    match &ast.node {
        Node::Match { cond, cases } => {
            let (cond_ty, mut env) = infer(cond, env, ctx, state)?;
            let state = state.in_match(&cond_ty.expr_ty(&cond.pos)?);

            let mut ty: Option<InferType> = None;
            for case in cases {
                let (case_ty, new_env) = infer(case, &env, ctx, &state)?;
                env = new_env;
                ty = if let Some(ty) = ty {
                    Some(ty.union(&case_ty, &case.pos)?)
                } else {
                    Some(case_ty)
                };
            }

            match ty {
                Some(ty) => Ok((ty, env)),
                None => Err(vec![TypeErr::new(&ast.pos, "Match must have arms")])
            }
        }
        Node::Case { cond, body } => {
            // TODO expand so we except more than just literals and identifiers
            let match_ty = state
                .in_match
                .clone()
                .ok_or(vec![TypeErr::new(&ast.pos, "Case cannot be outside match")])?;
            let mut env = env.clone();

            // TODO make first item of IdType String instead of another AST
            // TODO handle cases where identifier is a class
            let cond_ty = match &cond.node {
                Node::IdType { .. } | Node::Id { .. } | Node::Tuple { .. } => {
                    let identifier = Identifier::try_from(cond.deref())?;
                    for (id, (mutable, expr_ty)) in
                        match_name(&identifier, &match_ty, state, &cond.pos)?
                    {
                        env.insert(&id, mutable, &expr_ty);
                    }
                    InferType::from(&match_ty)
                }
                Node::Str { .. } | Node::Int { .. } | Node::Real { .. } | Node::Bool { .. } =>
                    infer(&cond, &env, ctx, state)?.0,
                _ =>
                    return Err(vec![TypeErr::new(
                        &cond.pos,
                        "Currently, condition may only be identifier or literal"
                    )]),
            };

            cond_ty.expr_ty(&cond.pos)?;
            infer(body, &env, ctx, state)
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected control flow")])
    }
}
