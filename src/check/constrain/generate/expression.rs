use std::convert::TryFrom;

use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::expected::{Expect, Expected};
use crate::check::constrain::generate::{Constrained, generate};
use crate::check::constrain::generate::definition::{constrain_args, identifier_from_var};
use crate::check::constrain::generate::env::Environment;
use crate::check::context::Context;
use crate::check::result::TypeErr;
use crate::parse::ast::{AST, Node, OptAST};

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
        Node::ExpressionType { expr, mutable, ty } =>
            match_id(expr, ty, *mutable, env, ctx, constr),
        Node::Id { .. } => match_id(ast, &None, false, env, ctx, constr),
        Node::Question { left, right } => {
            constr.add(
                "question",
                &Expected::try_from((left, &env.var_mappings))?,
                &Expected::new(left.pos, &Expect::none()),
            );

            generate(left, env, ctx, constr)?;
            generate(right, &env, ctx, constr)?;
            Ok(env.clone())
        }
        Node::Pass => if let Some(expected_ret_ty) = &env.return_type {
            constr.add("pass", &Expected::new(ast.pos, &Expect::none()), expected_ret_ty);
            Ok(env.clone())
        } else {
            Ok(env.clone())
        },

        _ => Err(vec![TypeErr::new(ast.pos, "Expected an expression")])
    }
}

fn match_id(ast: &AST, ty: &OptAST, mutable: bool, env: &Environment, ctx: &Context, constr: &mut ConstrBuilder) -> Constrained {
    match &ast.node {
        Node::Id { lit } => if env.is_def_mode {
            identifier_from_var(ast, ty, &None, mutable, ctx, constr, env)
        } else if env.get_var(lit).is_some() {
            Ok(env.clone())
        } else {
            Err(vec![TypeErr::new(ast.pos, &format!("Undefined variable: {}", lit))])
        }
        _ => Ok(env.clone())
    }
}
