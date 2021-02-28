use std::convert::TryFrom;

use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::expected::Expect::*;
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::constraint::Constraint;
use crate::check::constrain::generate::collection::gen_collection_lookup;
use crate::check::constrain::generate::generate;
use crate::check::constrain::Constrained;
use crate::check::context::name::NameUnion;
use crate::check::context::{clss, Context};
use crate::check::env::Environment;
use crate::check::result::TypeErr;
use crate::parse::ast::{Node, AST};

pub fn gen_flow(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder
) -> Constrained {
    match &ast.node {
        Node::Handle { expr_or_stmt, .. } => {
            let mut res = (constr.clone(), env.clone());

            // TODO check that all raises are covered

            generate(expr_or_stmt, &res.1, ctx, &mut res.0)
        }

        Node::IfElse { cond, then, el: Some(el) } => {
            constr.new_set(true);
            let left = Expected::try_from((cond, env))?;
            constr.add_constr(&Constraint::truthy("if else", &left));
            let (mut constr, env) = generate(cond, env, ctx, constr)?;

            let left = Expected::try_from((then, &env))?;
            let right = Expected::try_from((el, &env))?;
            constr.add("if else", &left, &right);

            constr.new_set(true);
            let (mut constr, then_env) = generate(then, &env, ctx, &mut constr)?;
            constr.exit_set(&then.pos)?;
            constr.new_set(true);
            let (mut constr, else_env) = generate(el, &env, ctx, &mut constr)?;
            constr.exit_set(&el.pos)?;

            // TODO apply union to constraints
            constr.exit_set(&ast.pos)?;
            Ok((constr, env.union(&then_env.intersect(&else_env))))
        }
        Node::IfElse { cond, then, .. } => {
            constr.new_set(true);
            let left = Expected::try_from((cond, env))?;
            constr.add_constr(&Constraint::truthy("if else", &left));
            let (mut constr, env) = generate(cond, env, ctx, constr)?;

            let (mut constr, _) = generate(then, &env, ctx, &mut constr)?;
            constr.exit_set(&then.pos)?;
            Ok((constr, env))
        }

        Node::Case { .. } => Err(vec![TypeErr::new(&ast.pos, "Case cannot be top level")]),
        Node::Match { cond, cases } => {
            let mut res = (constr.clone(), env.clone());
            // TODO check that all variants are covered

            for case in cases {
                match &case.node {
                    Node::Case { cond, body } => {
                        res.0.add(
                            "match case",
                            &Expected::try_from((cond, env))?,
                            &Expected::try_from((cond, env))?
                        );
                        res.0.add(
                            "match body",
                            &Expected::try_from((body, env))?,
                            &Expected::try_from((ast, env))?
                        );
                        res = generate(body, &res.1, ctx, &mut res.0)?;
                    }
                    _ => return Err(vec![TypeErr::new(&case.pos, "Expected case")])
                }
            }

            generate(cond, &res.1, ctx, &mut res.0)
        }

        Node::For { expr, col, body } => {
            constr.new_set(true);
            let (mut constr, for_env) = generate(col, &env, ctx, constr)?;

            let is_define_mode = for_env.is_define_mode;
            let (mut constr, for_env) =
                gen_collection_lookup(expr, &col, &for_env.define_mode(true), &mut constr)?;
            let (mut constr, _) =
                generate(body, &for_env.in_loop().define_mode(is_define_mode), ctx, &mut constr)?;

            constr.exit_set(&ast.pos)?;
            Ok((constr, env.clone()))
        }
        Node::Step { amount } => {
            let name = NameUnion::from(clss::INT_PRIMITIVE);
            constr.add(
                "step",
                &Expected::try_from((amount, env))?,
                &Expected::new(&amount.pos, &Type { name })
            );
            Ok((constr.clone(), env.clone()))
        }
        Node::While { cond, body } => {
            constr.new_set(true);
            let left = Expected::try_from((cond, env))?;
            constr.add_constr(&Constraint::truthy("if else", &left));
            let (mut constr, env) = generate(cond, &env, ctx, constr)?;
            let (mut constr, _) = generate(body, &env.in_loop(), ctx, &mut constr)?;
            constr.exit_set(&ast.pos)?;
            Ok((constr, env))
        }

        Node::Break | Node::Continue =>
            if env.in_loop {
                Ok((constr.clone(), env.clone()))
            } else {
                Err(vec![TypeErr::new(&ast.pos, "Cannot be outside loop")])
            },

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected control flow")])
    }
}
