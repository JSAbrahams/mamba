use crate::parser::ast::{Node, AST};
use crate::type_checker::checker_result::TypeErr;
use crate::type_checker::constraints::constraint::builder::ConstrBuilder;
use crate::type_checker::constraints::constraint::expected::Expect::*;
use crate::type_checker::constraints::constraint::expected::Expected;
use crate::type_checker::constraints::generate::definition::constrain_args;
use crate::type_checker::constraints::generate::generate;
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;

pub fn gen_expr(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder
) -> Constrained {
    match &ast.node {
        Node::AnonFun { args, body } => {
            // TODO generate constraint for anonymous function itself
            let (mut constr, env) = constrain_args(args, env, ctx, constr)?;
            generate(body, &env, ctx, &mut constr)
        }
        Node::Id { lit } if env.get_var(lit).is_some() => Ok((constr.clone(), env.clone())),
        Node::Id { lit } =>
            Err(vec![TypeErr::new(&ast.pos, &format!("Undefined variable: {}", lit))]),
        Node::Question { left, right } => {
            constr.add(&Expected::from(left), &Expected::new(&left.pos, &Nullable));
            let (mut constr, env) = generate(left, env, ctx, constr)?;
            generate(right, &env, ctx, &mut constr)
        }
        Node::Pass =>
            if let Some(expected_ret_ty) = &env.return_type {
                if env.last_stmt_in_function {
                    constr.add(&Expected::new(&ast.pos, &Nullable), &expected_ret_ty);
                }
                Ok((constr.clone(), env.clone()))
            } else {
                Ok((constr.clone(), env.clone()))
            },

        Node::Undefined => {
            constr.add(&Expected::from(ast), &Expected::new(&ast.pos, &Nullable));
            Ok((constr.clone(), env.clone()))
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected an expression")])
    }
}
