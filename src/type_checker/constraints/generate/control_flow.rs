use std::convert::TryFrom;
use std::ops::Deref;

use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::cons::Expect::{Expression, ExpressionAny, Raises, RaisesAny,
                                                     Truthy, Type};
use crate::type_checker::constraints::cons::{Constraints, Expect};
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
                            if let Some(ty) = ty {
                                let type_name = TypeName::try_from(ty.deref())?;
                                res.0 = res.0.add(&Raises { type_name }, &cond_expect);
                            } else {
                                res.0 = res.0.add(&RaisesAny, &cond_expect)
                            }
                            if let Some(first_case_exp) = &first_case_exp {
                                res.0 =
                                    res.0.add(first_case_exp, &Expression { ast: *body.clone() });
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
            let constr = if env.state.expect_expr {
                constr.add(&Expression { ast: *then.clone() }, &Expression { ast: *el.clone() })
            } else {
                constr.clone()
            }
            .add(&Expression { ast: *cond.clone() }, &Truthy);
            let (constr, env) = generate(cond, env, ctx, &constr)?;
            let (constr, _) = generate(then, &env, ctx, &constr)?;
            let (constr, _) = generate(el, &env, ctx, &constr)?;
            Ok((constr, env))
        }
        Node::IfElse { cond, then, .. } => {
            let constr = constr.add(&Expression { ast: *cond.clone() }, &Truthy);
            let constr = if env.state.expect_expr {
                constr.add(&Expression { ast: *then.clone() }, &ExpressionAny)
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
                        res.0 = res.0.add(&Expression { ast: *cond.clone() }, &cond_expect);
                        if let Some(case_expectation) = &first_case_exp {
                            res.0 = res.0.add(case_expectation, &Expression { ast: *body.clone() });
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
            let constr = constr.add(&Expression { ast: *amount.clone() }, &Type { type_name });
            Ok((constr, env.clone()))
        }
        Node::While { cond, body } => {
            let constr = constr.add(&Expression { ast: *cond.clone() }, &Truthy);
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
