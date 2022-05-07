use std::convert::TryFrom;

use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::expected::{Expect, Expected};
use crate::check::constrain::generate::{Constrained, generate};
use crate::check::constrain::generate::definition::{constrain_args, identifier_from_var};
use crate::check::constrain::generate::env::Environment;
use crate::check::context::Context;
use crate::check::result::TypeErr;
use crate::parse::ast::{AST, Node};

pub fn gen_expr(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    match &ast.node {
        Node::AnonFun { args, body } => {
            // TODO generate constraint for anonymous function itself
            let (mut constr, env) = constrain_args(args, env, ctx, constr)?;
            generate(body, &env, ctx, &mut constr)
        }

        Node::Id { .. }  if env.is_define_mode => identifier_from_var(ast, &None, &None, false, ctx, constr, env),
        Node::Id { lit } if env.get_var(lit).is_some() => Ok((constr.clone(), env.clone())),
        Node::Id { lit } => Err(vec![TypeErr::new(&ast.pos, &format!("Undefined variable: {}", lit))]),

        Node::Question { left, right } => {
            constr.add(
                "question",
                &Expected::try_from((left, &env.var_mappings))?,
                &Expected::new(&left.pos, &Expect::none()),
            );
            let (mut constr, env) = generate(left, env, ctx, constr)?;
            generate(right, &env, ctx, &mut constr)
        }
        Node::Pass => if let Some(expected_ret_ty) = &env.return_type {
            if env.last_stmt_in_function {
                constr.add("pass", &Expected::new(&ast.pos, &Expect::none()), expected_ret_ty);
            }
            Ok((constr.clone(), env.clone()))
        } else {
            Ok((constr.clone(), env.clone()))
        },

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected an expression")])
    }
}
