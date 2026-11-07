use std::convert::TryFrom;

use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::constraint::{Constraint, MapExp};
use crate::check::constrain::generate::collection::constr_col_lookup;
use crate::check::constrain::generate::env::Environment;
use crate::check::constrain::generate::{generate, Constrained};
use crate::check::context::Context;
use crate::check::name::true_name::TrueName;
use crate::check::result::TypeErr;
use crate::parse::ast::{Node, AST};

#[allow(clippy::result_large_err)]
pub fn gen_flow(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    match &ast.node {
        Node::Handle {
            expr_or_stmt,
            cases,
        } => {
            let (raises, errs): (Vec<Result<_, _>>, Vec<Result<_, _>>) = cases
                .iter()
                .map(|c| match &c.node {
                    // A handle case becomes a Python `except`, which has nowhere to put a guard.
                    Node::Case {
                        guard: Some(guard), ..
                    } => Err(TypeErr::new(guard.pos, "Handle case cannot have a guard")),
                    Node::Case { cond, .. } => match &cond.node {
                        Node::ExpressionType { ty: Some(ty), .. } => TrueName::try_from(ty)
                            .map_err(|errs| errs.first().expect("At least one").clone()),
                        other => {
                            let msg = format!("Expected type identifier, was {other}");
                            Err(TypeErr::new(cond.pos, &msg))
                        }
                    },
                    other => Err(TypeErr::new(c.pos, &format!("Expected case, was {other}"))),
                })
                .partition(Result::is_ok);

            if !errs.is_empty() {
                return Err(errs.into_iter().map(Result::unwrap_err).collect());
            }
            let raises = raises.into_iter().map(Result::unwrap).collect();

            let raises_before = env.raises_caught.clone();
            let outer_env = generate(expr_or_stmt, &env.raises_caught(&raises), ctx, constr)?
                .raises_caught(&raises_before);

            constrain_cases(ast, &None, cases, &outer_env, ctx, constr)
        }

        Node::IfElse {
            cond,
            then,
            el: Some(el),
        } => {
            constr.add_constr(
                &Constraint::truthy("if condition", &Expected::from(cond)),
                env,
            );
            generate(cond, env, ctx, constr)?;
            let if_expr_exp = Expected::from(ast);

            constr.branch_point();
            generate(then, env, ctx, constr)?;
            if env.is_expr {
                constr.add(
                    "then branch equal to if",
                    &if_expr_exp,
                    &Expected::from(then),
                    env,
                );
            }

            constr.branch("if else branch", el.pos);
            generate(el, env, ctx, constr)?;
            if env.is_expr {
                constr.add(
                    "else branch equal to if",
                    &if_expr_exp,
                    &Expected::from(el),
                    env,
                );
            }

            constr.reset_branches();
            Ok(env.clone())
        }
        Node::IfElse { cond, then, .. } => {
            constr.add_constr(
                &Constraint::truthy("if condition", &Expected::from(cond)),
                env,
            );

            generate(cond, env, ctx, constr)?;
            generate(then, env, ctx, constr)?;
            Ok(env.clone())
        }

        Node::Case { .. } => Err(vec![TypeErr::new(ast.pos, "Case cannot be top level")]),
        Node::Match { cond, cases } => {
            cases.iter().try_for_each(check_no_annotation)?;
            let outer_env = generate(cond, env, ctx, constr)?;
            constrain_cases(ast, &Some(*cond.clone()), cases, &outer_env, ctx, constr)
        }

        Node::For { expr, col, body } => {
            let col_env = generate(col, env, ctx, constr)?;
            let lookup_env = constr_col_lookup(expr, col, &col_env.is_def_mode(true), constr)?
                .is_def_mode(false);
            let lookup_env = generate(expr, &lookup_env, ctx, constr)?;

            generate(body, &lookup_env.in_loop(), ctx, constr)?;
            Ok(env.clone())
        }
        Node::While { cond, body } => {
            constr.add_constr(
                &Constraint::truthy("while condition", &Expected::from(cond)),
                env,
            );

            generate(cond, env, ctx, constr)?;
            generate(body, &env.in_loop(), ctx, constr)?;
            Ok(env.clone())
        }

        Node::Break | Node::Continue if env.in_loop => Ok(env.clone()),
        Node::Break | Node::Continue => Err(vec![TypeErr::new(ast.pos, "Cannot be outside loop")]),

        _ => Err(vec![TypeErr::new(ast.pos, "Expected control flow")]),
    }
}

fn constrain_cases(
    ast: &AST,
    expr: &Option<AST>,
    cases: &Vec<AST>,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    let is_define_mode = env.is_def_mode;
    constr.branch_point();

    let mut envs = vec![];
    for case in cases {
        match &case.node {
            Node::Case { cond, guard, body } => {
                constr.branch("match arm", case.pos);
                check_no_rest(cond)?;
                let cond_env = generate(cond, &env.is_def_mode(true), ctx, constr)?;
                tie_pattern_to_subject(expr, cond, env, &cond_env, constr);

                // Generated after the pattern is tied to the subject, and in the environment the
                // pattern produced, so the guard can read what the pattern bound.
                if let Some(guard) = guard {
                    let guard_env = cond_env.is_def_mode(false);
                    constr.add_constr(
                        &Constraint::truthy("match arm guard", &Expected::from(guard)),
                        &guard_env,
                    );
                    generate(guard, &guard_env, ctx, constr)?;
                }

                let body_env = generate(body, &cond_env.is_def_mode(is_define_mode), ctx, constr)?;
                let (exp_body, exp_match) = (Expected::from(body), Expected::from(ast));
                add_across_envs(
                    "arm body",
                    (&exp_body, &body_env),
                    (&exp_match, env),
                    constr,
                );
                if env.is_expr {
                    add_across_envs(
                        "arm body and outer",
                        (&exp_match, env),
                        (&exp_body, &body_env),
                        constr,
                    );
                }
                envs.push(body_env);
            }
            _ => return Err(vec![TypeErr::new(case.pos, "Expected case")]),
        }
    }

    constr.reset_branches();

    Ok(env.clone())
}

/// Give a match pattern the type of the subject it is matched against.
///
/// The subject is read in the outer environment and the pattern in the one it binds into.
/// A pattern name that shadows an outer one, such as `(m, n)` in a function taking `m` and `n`, then refers to the new binding.
fn tie_pattern_to_subject(
    subject: &Option<AST>,
    cond: &AST,
    env: &Environment,
    cond_env: &Environment,
    constr: &mut ConstrBuilder,
) {
    let (Some(subject), Node::ExpressionType { expr: pattern, .. }) = (subject, &cond.node) else {
        return;
    };

    let (subject, pattern) = (Expected::from(subject), Expected::from(pattern.as_ref()));
    add_across_envs(
        "match pattern",
        (&subject, env),
        (&pattern, cond_env),
        constr,
    );
}

/// Constrain `parent >= child`, reading each side's names in its own environment.
///
/// [ConstrBuilder::add] maps both sides through one environment, so each side is mapped up front instead.
fn add_across_envs(
    msg: &str,
    (parent, parent_env): (&Expected, &Environment),
    (child, child_env): (&Expected, &Environment),
    constr: &mut ConstrBuilder,
) {
    let parent = parent.map_exp(&parent_env.var_mapping, &constr.var_mapping);
    let child = child.map_exp(&child_env.var_mapping, &constr.var_mapping);
    constr.add_constr_map(
        &Constraint::new(msg, &parent, &child),
        &parent_env.var_mapping,
        true,
    );
}

/// Reject a type annotation on a match case.
///
/// Only a handle case gives it a meaning, the exception type it catches.
fn check_no_annotation(case: &AST) -> Constrained<()> {
    let Node::Case { cond, .. } = &case.node else {
        return Ok(());
    };
    let Node::ExpressionType { ty: Some(ty), .. } = &cond.node else {
        return Ok(());
    };

    Err(vec![TypeErr::new(
        ty.pos,
        "A match case cannot have a type annotation",
    )])
}

/// Reject a `..` in a match case, which is where it is meant to go next.
///
/// Allowing `(2, ..)` later means matching on the elided elements here.
fn check_no_rest(ast: &AST) -> Constrained<()> {
    match &ast.node {
        Node::Rest => Err(vec![TypeErr::new(
            ast.pos,
            "'..' is only allowed as the argument list of a bodiless 'new'",
        )]),
        Node::Tuple { elements } => elements.iter().try_for_each(check_no_rest),
        Node::ExpressionType { expr, .. } => check_no_rest(expr),
        _ => Ok(()),
    }
}
