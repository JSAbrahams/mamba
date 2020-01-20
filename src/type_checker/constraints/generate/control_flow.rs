use std::convert::TryFrom;
use std::ops::Deref;

use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::constraint::expected::Expect::*;
use crate::type_checker::constraints::constraint::expected::{Expect, Expected};
use crate::type_checker::constraints::constraint::Constraints;
use crate::type_checker::constraints::generate::collection::constrain_collection;
use crate::type_checker::constraints::generate::generate;
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::{ty, Context};
use crate::type_checker::environment::Environment;
use crate::type_checker::type_name::TypeName;
use crate::type_checker::type_result::TypeErr;

pub fn gen_flow(ast: &AST, env: &Environment, ctx: &Context, constr: &Constraints) -> Constrained {
    match &ast.node {
        Node::Handle { expr_or_stmt, cases } => {
            let mut res = (constr.clone(), env.clone());
            let first_case_exp = first_case_expect(cases, env);
            let cond_expect = Expression { ast: *expr_or_stmt.clone() };

            // TODO check that all raises are covered
            for case in cases {
                match &case.node {
                    Node::Case { cond: case_cond, body } => match &case_cond.node {
                        Node::ExpressionType { expr: _, mutable: _, ty } => {
                            // TODO use expr and mutable
                            let (left, right) = if let Some(ty) = ty {
                                let type_name = TypeName::try_from(ty.deref())?;
                                (
                                    Expected::new(&body.pos, &Raises { type_name }),
                                    Expected::new(&body.pos, &cond_expect)
                                )
                            } else {
                                (
                                    Expected::new(&body.pos, &RaisesAny),
                                    Expected::new(&body.pos, &cond_expect)
                                )
                            };
                            res.0 = res.0.add(&left, &right);

                            if let Some(first_case) = &first_case_exp {
                                let left = Expected::new(&case.pos, first_case);
                                let right =
                                    Expected::new(&case.pos, &Expression { ast: *body.clone() });
                                res.0 = res.0.add(&left, &right);
                            }
                            res = generate(body, &res.1, ctx, &res.0)?;
                        }
                        _ =>
                            return Err(vec![TypeErr::new(
                                &case_cond.pos,
                                "Expected expression type"
                            )]),
                    },
                    _ => return Err(vec![TypeErr::new(&case.pos, "Expected case")])
                }
            }

            generate(expr_or_stmt, &res.1, ctx, &res.0)
        }

        Node::IfElse { cond, then, el: Some(el) } => {
            let left = Expected::new(&cond.pos, &Expression { ast: *cond.clone() });
            let constr = constr.add(&left, &Expected::new(&cond.pos, &Truthy));

            let constr = if env.state.expect_expr {
                let left = Expected::new(&then.pos, &Expression { ast: *then.clone() });
                let right = Expected::new(&el.pos, &Expression { ast: *el.clone() });
                constr.add(&left, &right)
            } else {
                constr.clone()
            };

            let (constr, env) = generate(cond, env, ctx, &constr)?;
            let (constr, _) = generate(then, &env, ctx, &constr)?;
            let (constr, _) = generate(el, &env, ctx, &constr)?;
            Ok((constr, env))
        }
        Node::IfElse { cond, then, .. } => {
            let left = Expected::new(&cond.pos, &Expression { ast: *cond.clone() });
            let constr = constr.add(&left, &Expected::new(&cond.pos, &Truthy));

            let constr = if env.state.expect_expr {
                let left = Expected::new(&then.pos, &Expression { ast: *then.clone() });
                constr.add(&left, &Expected::new(&then.pos, &ExpressionAny))
            } else {
                constr
            };

            let (constr, env) = generate(cond, env, ctx, &constr)?;
            let (constr, _) = generate(then, &env, ctx, &constr)?;
            Ok((constr, env))
        }

        Node::Case { .. } => Err(vec![TypeErr::new(&ast.pos, "Case cannot be top level")]),
        Node::Match { cond, cases } => {
            let mut res = (constr.clone(), env.clone());
            let cond_expect = Expression { ast: *cond.clone() };
            let first_case_exp = first_case_expect(cases, env);

            // TODO check that all variants are covered
            for case in cases {
                match &case.node {
                    Node::Case { cond, body } => {
                        let left = Expected::new(&cond.pos, &Expression { ast: *cond.clone() });
                        res.0 = res.0.add(&left, &Expected::new(&cond.pos, &cond_expect));

                        if let Some(case_expectation) = &first_case_exp {
                            let left = Expected::new(&case.pos, case_expectation);
                            let right =
                                Expected::new(&case.pos, &Expression { ast: *body.clone() });
                            res.0 = res.0.add(&left, &right);
                        }
                        res = generate(body, &res.1, ctx, &res.0)?;
                    }
                    _ => return Err(vec![TypeErr::new(&case.pos, "Expected case")])
                }
            }

            generate(cond, &res.1, ctx, &res.0)
        }

        Node::For { expr, col, body } => {
            let (constr, env) = constrain_collection(col, expr, env, ctx, constr)?;
            let (constr, _) = generate(body, &env, ctx, &constr)?;
            Ok((constr, env.clone()))
        }
        Node::Step { amount } => {
            let type_name = TypeName::from(ty::concrete::INT_PRIMITIVE);
            let left = Expected::new(&amount.pos, &Expression { ast: *amount.clone() });
            let constr = constr.add(&left, &Expected::new(&amount.pos, &Type { type_name }));
            Ok((constr, env.clone()))
        }
        Node::While { cond, body } => {
            let left = Expected::new(&cond.pos, &Expression { ast: *cond.clone() });
            let constr = constr.add(&left, &Expected::new(&cond.pos, &Truthy));
            let (constr, _) = generate(body, env, ctx, &constr)?;
            Ok((constr, env.clone()))
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected control flow")])
    }
}

fn first_case_expect(cases: &Vec<AST>, env: &Environment) -> Option<Expect> {
    if let Some(case) = cases.first() {
        if env.state.expect_expr {
            Some(Expression { ast: case.clone() })
        } else {
            None
        }
    } else {
        None
    }
}
