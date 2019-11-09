use std::convert::TryFrom;
use std::ops::Deref;

use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::context::Context;
use crate::type_checker::environment::expression_type::ExpressionType;
use crate::type_checker::environment::infer_type::InferType;
use crate::type_checker::environment::name::{match_name, Identifier};
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::TypeErr;

// TODO add pattern matching type checking

pub fn infer_match(ast: &AST, env: &Environment, ctx: &Context) -> InferResult {
    match &ast.node {
        Node::Match { cond, cases } => {
            let (cond_ty, env) = infer(cond, env, ctx)?;
            let state = env.state.in_match(&cond_ty.expr_ty(&cond.pos)?);
            vec_to_union(cases, &env.new_state(&state), ctx, &ast.pos)
        }
        Node::Handle { expr_or_stmt, cases } => {
            let (cond_ty, env) = infer(expr_or_stmt, env, ctx)?;
            let state = env.state.handling(&cond_ty.raises.into_iter().collect());
            vec_to_union(cases, &env.new_state(&state), ctx, &ast.pos)
        }

        Node::Case { cond, body } => {
            let match_ty = remaining_match_ty(ast, cond, env, ctx)?;
            let mut env = env.clone();

            // TODO expand so we accept more than just literals and identifiers
            // TODO handle cases where identifier is a class
            // TODO treat identifier without type as default
            // TODO check that _type is covered by match_ty
            match &cond.node {
                Node::IdType { id, mutable, _type } => {
                    if let Some(_type) = _type {
                        let type_name = TypeName::try_from(_type.deref())?;
                        ctx.lookup(&type_name, &_type.pos)?;
                    }

                    match &id.node {
                        Node::Str { .. }
                        | Node::Int { .. }
                        | Node::Real { .. }
                        | Node::Bool { .. } => {
                            infer(&id, &env, ctx)?.0.expr_ty(&cond.pos)?;
                        }
                        Node::Underscore => {}
                        _ => {
                            let identifier = Identifier::try_from(cond.deref())?;
                            let matched = match_name(&identifier, &match_ty, &env, &cond.pos)?;
                            for (id, (inner_mut, expr_ty)) in matched {
                                env.insert(&id, *mutable || inner_mut, &expr_ty);
                            }
                        }
                    }
                }
                _ => return Err(vec![TypeErr::new(&cond.pos, "Expected expression maybe type")])
            };

            infer(body, &env, ctx)
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected control flow")])
    }
}

fn remaining_match_ty(
    ast: &AST,
    cond: &AST,
    env: &Environment,
    ctx: &Context
) -> Result<ExpressionType, Vec<TypeErr>> {
    if let Some(in_match) = &env.state.in_match {
        Ok(in_match.clone())
    } else if !env.state.handling.is_empty() {
        let mut handling: Option<TypeName> = None;
        for handle in &env.state.handling {
            handling = match handling {
                None => Some(TypeName::from(handle)),
                Some(handling) => Some(handling.union(&TypeName::from(handle)))
            };
        }

        if let Some(handling) = handling {
            ctx.lookup(&handling, &cond.pos)
        } else {
            Err(vec![TypeErr::new(&cond.pos, "Error never thrown")])
        }
    } else {
        Err(vec![TypeErr::new(&ast.pos, "Case cannot be outside match")])
    }
}

fn vec_to_union(cases: &Vec<AST>, env: &Environment, ctx: &Context, pos: &Position) -> InferResult {
    let mut ty: Option<InferType> = None;
    for case in cases {
        let (case_ty, _) = infer(case, &env, ctx)?;
        ty = if let Some(ty) = ty { Some(ty.union(&case_ty, &case.pos)?) } else { Some(case_ty) };
    }

    ty.map(|ty| (ty, env.clone())).ok_or(vec![TypeErr::new(pos, "Arms are empty")])
}
