use std::convert::TryFrom;

use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::generate::definition::{constrain_args, id_from_var};
use crate::check::constrain::generate::env::Environment;
use crate::check::constrain::generate::{generate, Constrained};
use crate::check::context::Context;
use crate::check::name::Name;
use crate::check::result::TypeErr;
use crate::parse::ast::{Node, OptAST, AST};

pub fn gen_expr(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    match &ast.node {
        Node::AnonFun { args, body } => {
            let anon_env = constrain_args(args, &env.is_def_mode(true), ctx, constr)?;
            generate(body, &anon_env, ctx, constr)?;
            Ok(env.clone())
        }
        Node::ExpressionType { expr, mutable, ty } => {
            match_id(expr, ty, *mutable, env, ctx, constr)
        }
        Node::Id { .. } => match_id(ast, &None, false, env, ctx, constr),
        Node::Question { left, right } => {
            constr.add("question", &Expected::from(left), &Expected::none(left.pos), env);

            generate(left, env, ctx, constr)?;
            generate(right, env, ctx, constr)?;
            Ok(env.clone())
        }
        Node::Pass => {
            if let Some(expected_ret_ty) = &env.return_type {
                constr.add("pass", &Expected::none(ast.pos), expected_ret_ty, env);
                Ok(env.clone())
            } else {
                Ok(env.clone())
            }
        }

        _ => Err(vec![TypeErr::new(ast.pos, "Expected an expression")]),
    }
}

fn match_id(
    ast: &AST,
    ty: &OptAST,
    mutable: bool,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    match &ast.node {
        Node::Id { lit } => {
            if env.is_def_mode {
                let ty = if let Some(ty) = ty { Some(Name::try_from(ty)?) } else { None };
                id_from_var(ast, &ty, &None, mutable, ctx, constr, env)
            } else if env.is_destruct_mode {
                Ok(env.remove_var(lit))
            } else if env.get_var(lit, &constr.var_mapping).is_some() {
                Ok(env.clone())
            } else {
                Err(vec![TypeErr::new(ast.pos, &format!("Undefined variable: {lit}"))])
            }
        }
        _ => Ok(env.clone()),
    }
}
