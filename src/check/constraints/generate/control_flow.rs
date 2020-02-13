use crate::check::checker_result::TypeErr;
use crate::check::constraints::constraint::builder::ConstrBuilder;
use crate::check::constraints::constraint::expected::Expect::*;
use crate::check::constraints::constraint::expected::Expected;
use crate::check::constraints::generate::collection::{constr_col, gen_collection_lookup};
use crate::check::constraints::generate::generate;
use crate::check::constraints::Constrained;
use crate::check::context::{ty, Context};
use crate::check::environment::Environment;
use crate::check::ty::name::TypeName;
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
            constr.exit_set(&ast.pos)?;
            Ok((constr, env.union(&then_env.intersect(&else_env))))
        }
        Node::IfElse { cond, then, .. } => {
            constr.new_set(true);
            let left = Expected::from(cond);
            constr.add(&left, &Expected::new(&cond.pos, &Truthy));
            let (mut constr, env) = generate(cond, env, ctx, constr)?;

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
            constr.new_set(true);
            let mut constr = constr_col(col, constr);
            let (mut constr, env) =
                gen_collection_lookup(expr, &col, &env.define_mode(), &mut constr)?;
            let (mut constr, env) = generate(col, &env, ctx, &mut constr)?;
            let (mut constr, _) = generate(body, &env.in_loop(), ctx, &mut constr)?;
            constr.exit_set(&ast.pos)?;
            Ok((constr, env))
        }
        Node::Step { amount } => {
            let left = Expected::from(amount);
            let type_name = TypeName::from(ty::concrete::INT_PRIMITIVE);
            constr.add(&left, &Expected::new(&amount.pos, &Type { type_name }));
            Ok((constr.clone(), env.clone()))
        }
        Node::While { cond, body } => {
            constr.new_set(true);
            let left = Expected::from(cond);
            constr.add(&left, &Expected::new(&cond.pos, &Truthy));
            let (mut constr, env) = generate(cond, &env, ctx, constr)?;
            let (mut constr, _) = generate(body, &env.in_loop(), ctx, &mut constr)?;
            constr.exit_set(&ast.pos)?;
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
