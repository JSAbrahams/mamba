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
            let left = Expected::try_from((cond, &constr.var_mapping))?;
            constr.add_constr(&Constraint::truthy("if else", &left));
            generate(cond, env, ctx, constr)?;

            let if_expr_exp = Expected::try_from((ast, &constr.var_mapping))?;

            constr.new_set();
            let then_env = generate(then, env, ctx, constr)?;
            let then_exp = Expected::try_from((then, &constr.var_mapping))?;
            if env.is_expr {
                constr.add("then branch equal to if", &then_exp, &if_expr_exp);
            }

            constr.new_set();
            let else_env = generate(el, env, ctx, constr)?;
            if env.is_expr {
                let el = Expected::try_from((el, &constr.var_mapping))?;
                constr.add("else branch equal to if", &el, &then_exp);
            }

            Ok(env.union(&then_env.intersect(&else_env)))
        }
        Node::IfElse { cond, then, .. } => {
            let left = Expected::try_from((cond, &constr.var_mapping))?;
            constr.add_constr(&Constraint::truthy("if else", &left));

            generate(cond, env, ctx, constr)?;
            constr.new_set();
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
            let loop_lvl = constr.new_set();
            let col_env = generate(col, env, ctx, constr)?;

            let is_define_mode = col_env.is_def_mode;
            let lookup_env = gen_collection_lookup(expr, col, &col_env.is_def_mode(true), constr)?;

            generate(body, &lookup_env.in_loop().is_def_mode(is_define_mode), ctx, constr)?;
            constr.exit_set_to(loop_lvl, ast.pos)?;
            Ok(env.clone())
        }
        Node::While { cond, body } => {
            let while_lvl = constr.new_set();
            let cond_exp = Expected::try_from((cond, &constr.var_mapping))?;
            constr.add_constr(&Constraint::truthy("while condition", &cond_exp));

            generate(cond, env, ctx, constr)?;
            generate(body, &env.in_loop(), ctx, constr)?;
            constr.exit_set_to(while_lvl, ast.pos)?;
            Ok(env.clone())
        }

        Node::Break | Node::Continue if env.in_loop => Ok(env.clone()),
        Node::Break | Node::Continue => Err(vec![TypeErr::new(ast.pos, "Cannot be outside loop")]),

        _ => Err(vec![TypeErr::new(ast.pos, "Expected control flow")])
    }
}

fn constrain_cases(ast: &AST, expr: &Option<AST>, cases: &Vec<AST>, env: &Environment, ctx: &Context, constr: &mut ConstrBuilder) -> Constrained<()> {
    let is_define_mode = env.is_def_mode;
    let exp_ast = Expected::try_from((ast, &constr.var_mapping))?;

    for case in cases {
        match &case.node {
            Node::Case { cond, body } => {
                constr.new_set();

                let cond_env = generate(cond, &env.is_def_mode(true), ctx, constr)?;
                generate(body, &cond_env.is_def_mode(is_define_mode), ctx, constr)?;

                if let Node::ExpressionType { expr: ref cond, .. } = cond.node {
                    if let Some(expr) = expr {
                        constr.add("match expression and arm condition",
                                   &Expected::try_from((expr, &constr.var_mapping))?,
                                   &Expected::try_from((cond, &constr.var_mapping))?,
                        );
                    }
                }

                let exp_body = Expected::try_from((body, &constr.var_mapping))?;
                constr.add("match arm body", &exp_body, &exp_ast);
            }
            _ => return Err(vec![TypeErr::new(case.pos, "Expected case")])
        }
    }
    Ok(())
}
