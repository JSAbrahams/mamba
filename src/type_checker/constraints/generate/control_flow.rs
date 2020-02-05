use crate::parser::ast::{Node, AST};
use crate::type_checker::checker_result::TypeErr;
use crate::type_checker::constraints::constraint::builder::ConstrBuilder;
use crate::type_checker::constraints::constraint::expected::Expect::*;
use crate::type_checker::constraints::constraint::expected::Expected;
use crate::type_checker::constraints::generate::collection::{constr_col, gen_collection_lookup};
use crate::type_checker::constraints::generate::generate;
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::{ty, Context};
use crate::type_checker::environment::Environment;
use crate::type_checker::ty_name::TypeName;

pub fn gen_flow(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder
) -> Constrained {
    match &ast.node {
        Node::Handle { expr_or_stmt, cases } => {
            let mut res = (constr.clone(), env.clone());
            let cond_expect = Expression { ast: *expr_or_stmt.clone() };

            // TODO check that all raises are covered

            generate(expr_or_stmt, &res.1, ctx, &mut res.0)
        }

        Node::IfElse { cond, then, el: Some(el) } => {
            let left = Expected::from(cond);
            constr.add(&left, &Expected::new(&cond.pos, &Truthy));
            let (mut constr, env) = generate(cond, env, ctx, constr)?;

            let left = Expected::from(then);
            let right = Expected::from(el);
            constr.add(&left, &right);

            constr.new_set(true);
            let (mut constr, then_env) = generate(then, &env, ctx, &mut constr)?;
            constr.exit_set(&then.pos)?;
            constr.new_set(true);
            let (mut constr, else_env) = generate(el, &env, ctx, &mut constr)?;
            constr.exit_set(&el.pos)?;
            // TODO apply union to constraints
            Ok((constr, env.union(&then_env.intersect(&else_env))))
        }
        Node::IfElse { cond, then, .. } => {
            let left = Expected::from(cond);
            constr.add(&left, &Expected::new(&cond.pos, &Truthy));
            let (mut constr, env) = generate(cond, env, ctx, constr)?;

            constr.new_set(true);
            let (mut constr, _) = generate(then, &env, ctx, &mut constr)?;
            constr.exit_set(&then.pos)?;
            Ok((constr, env))
        }

        Node::Case { .. } => Err(vec![TypeErr::new(&ast.pos, "Case cannot be top level")]),
        Node::Match { cond, cases } => {
            let mut res = (constr.clone(), env.clone());
            let cond_exp = Expected::from(cond);

            // TODO check that all variants are covered
            for case in cases {
                match &case.node {
                    Node::Case { cond, body } => {
                        let left = Expected::from(cond);
                        res.0.add(&left, &cond_exp);
                        res = generate(body, &res.1, ctx, &mut res.0)?;
                    }
                    _ => return Err(vec![TypeErr::new(&case.pos, "Expected case")])
                }
            }

            generate(cond, &res.1, ctx, &mut res.0)
        }

        Node::For { expr, col, body } => {
            let (mut constr, col_exp) = constr_col(col, constr);
            let (mut constr, env) = gen_collection_lookup(expr, &col_exp, env, &mut constr)?;
            let (mut constr, env) = generate(col, &env, ctx, &mut constr)?;
            let (constr, _) = generate(body, &env.in_loop(), ctx, &mut constr)?;
            Ok((constr, env))
        }
        Node::Step { amount } => {
            let left = Expected::from(amount);
            let type_name = TypeName::from(ty::concrete::INT_PRIMITIVE);
            constr.add(&left, &Expected::new(&amount.pos, &Type { type_name }));
            Ok((constr.clone(), env.clone()))
        }
        Node::While { cond, body } => {
            let left = Expected::from(cond);
            constr.add(&left, &Expected::new(&cond.pos, &Truthy));
            let (mut constr, env) = generate(cond, &env, ctx, constr)?;
            let (constr, _) = generate(body, &env.in_loop(), ctx, &mut constr)?;
            Ok((constr, env.clone()))
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
