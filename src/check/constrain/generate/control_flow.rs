use std::convert::TryFrom;

use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::Constraint;
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::generate::{Constrained, generate};
use crate::check::constrain::generate::collection::gen_collection_lookup;
use crate::check::constrain::generate::env::Environment;
use crate::check::context::Context;
use crate::check::name::true_name::TrueName;
use crate::check::result::TypeErr;
use crate::parse::ast::{AST, Node};

pub fn gen_flow(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    match &ast.node {
        Node::Handle { expr_or_stmt, cases } => {
            let (raises, errs): (Vec<Result<_, _>>, Vec<Result<_, _>>) = cases.iter().map(|c| match &c.node {
                Node::Case { cond, .. } => {
                    match &cond.node {
                        Node::ExpressionType { ty: Some(ty), .. } => TrueName::try_from(ty)
                            .map_err(|errs| errs.first().expect("At least one").clone()),
                        other => {
                            let msg = format!("Expected type identifier, was {other}");
                            Err(TypeErr::new(cond.pos, &msg))
                        }
                    }
                }
                other => Err(TypeErr::new(c.pos, &format!("Expected case, was {other}")))
            }).partition(Result::is_ok);

            if !errs.is_empty() { return Err(errs.into_iter().map(Result::unwrap_err).collect()); }
            let raises = raises.into_iter().map(Result::unwrap).collect();

            let raises_before = env.raises_caught.clone();
            let outer_env = generate(expr_or_stmt, &env.raises_caught(&raises), ctx, constr)?
                .raises_caught(&raises_before);

            constrain_cases(ast, &None, cases, &outer_env, ctx, constr)?;
            Ok(outer_env.clone())
        }

        Node::IfElse { cond, then, el: Some(el) } => {
            constr.add_constr(&Constraint::truthy("if condition", &Expected::from(cond)), env);
            generate(cond, env, ctx, constr)?;
            let if_expr_exp = Expected::from(ast);

            constr.branch_point();
            generate(then, env, ctx, constr)?;
            if env.is_expr {
                constr.add("then branch equal to if", &Expected::from(then), &if_expr_exp, env);
            }

            constr.branch("if else branch", el.pos);
            generate(el, env, ctx, constr)?;
            if env.is_expr {
                constr.add("else branch equal to if", &Expected::from(el), &if_expr_exp, env);
            }

            constr.reset_branches();
            Ok(env.clone())
        }
        Node::IfElse { cond, then, .. } => {
            constr.add_constr(&Constraint::truthy("if condition", &Expected::from(cond)), env);

            generate(cond, env, ctx, constr)?;
            generate(then, env, ctx, constr)?;
            Ok(env.clone())
        }

        Node::Case { .. } => Err(vec![TypeErr::new(ast.pos, "Case cannot be top level")]),
        Node::Match { cond, cases } => {
            let outer_env = generate(cond, env, ctx, constr)?;
            constrain_cases(ast, &Some(*cond.clone()), cases, &outer_env, ctx, constr)?;
            Ok(env.clone())
        }

        Node::For { expr, col, body } => {
            let col_env = generate(col, env, ctx, constr)?;
            let lookup_env = gen_collection_lookup(expr, col, &col_env.is_def_mode(true), constr)?;

            generate(body, &lookup_env.in_loop().is_def_mode(false), ctx, constr)?;
            Ok(env.clone())
        }
        Node::While { cond, body } => {
            constr.add_constr(&Constraint::truthy("while condition", &Expected::from(cond)), env);

            generate(cond, env, ctx, constr)?;
            generate(body, &env.in_loop(), ctx, constr)?;
            Ok(env.clone())
        }

        Node::Break | Node::Continue if env.in_loop => Ok(env.clone()),
        Node::Break | Node::Continue => Err(vec![TypeErr::new(ast.pos, "Cannot be outside loop")]),

        _ => Err(vec![TypeErr::new(ast.pos, "Expected control flow")])
    }
}

fn constrain_cases(ast: &AST, expr: &Option<AST>, cases: &Vec<AST>, env: &Environment, ctx: &Context, constr: &mut ConstrBuilder) -> Constrained<()> {
    let is_define_mode = env.is_def_mode;
    constr.branch_point();

    for case in cases {
        match &case.node {
            Node::Case { cond, body } => {
                constr.branch("match arm", case.pos);
                let cond_env = generate(cond, &env.is_def_mode(true), ctx, constr)?;

                if let Node::ExpressionType { expr: ref cond, .. } = cond.node {
                    if let Some(expr) = &expr {
                        constr.add("arm body", &Expected::from(expr), &Expected::from(cond), env);
                    }
                }

                generate(body, &cond_env.is_def_mode(is_define_mode), ctx, constr)?;
                let exp_body = Expected::from(body);
                constr.add("arm body", &exp_body, &Expected::from(ast), env);

                if env.is_expr {
                    constr.add("arm body and outer", &Expected::from(ast), &exp_body, env);
                }
            }
            _ => return Err(vec![TypeErr::new(case.pos, "Expected case")])
        }
    }

    constr.reset_branches();
    Ok(())
}
