use std::collections::HashSet;

use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::cons::Constraints;
use crate::type_checker::constraints::cons::Expect::{Any, AnyExpr, Collection, Expression, Truthy,
                                                     Type};
use crate::type_checker::constraints::generate::generate;
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::{ty, Context};
use crate::type_checker::environment::Environment;
use crate::type_checker::type_name::TypeName;
use crate::type_checker::type_result::TypeErr;

pub fn gen_flow(ast: &AST, env: &Environment, ctx: &Context, constr: &Constraints) -> Constrained {
    match &ast.node {
        Node::Handle { expr_or_stmt, cases } => {
            let mut constr_env = (constr.clone(), env.clone());
            // TODO add system for generating constraints of thrown errors
            let mut raises: HashSet<TypeName> = unimplemented!();
            let case_body = if let Some(case) = cases.first() {
                match &expr_or_stmt.node {
                    Node::VariableDef { .. } | Node::Reassign { .. } =>
                        Some(Any { ast: case.clone() }),
                    _ => None
                }
            } else {
                None
            };

            // TODO add system for checking that all raises are covered
            let raises: TypeName = unimplemented!();
            for case in cases {
                match &case.node {
                    Node::Case { cond: case_cond, body } => {
                        constr_env.0 = constr_env.0.add(&Type { type_name: raises }, &Expression {
                            ast: *case_cond.clone()
                        });
                        if let Some(case_body) = &case_body {
                            constr_env.0 = constr_env.0.add(case_body, &Any { ast: *body.clone() });
                        }
                        constr_env = generate(body, &constr_env.1, ctx, &constr_env.0)?;
                    }
                    _ => return Err(vec![TypeErr::new(&case.pos, "Expected case")])
                }
            }

            generate(expr_or_stmt, &constr_env.1, ctx, &constr_env.0)
        }

        Node::IfElse { cond, then, el: Some(el) } => {
            let constr = if env.state.expect_expr {
                constr.add(&Expression { ast: *then.clone() }, &Expression { ast: *el.clone() })
            } else {
                constr.clone()
            }
            .add(&Expression { ast: *cond.clone() }, &Truthy);
            let (constr, env) = generate(cond, env, ctx, &constr)?;
            let (constr, env) = generate(then, &env, ctx, &constr)?;
            generate(el, &env, ctx, &constr)
        }
        Node::IfElse { cond, then, .. } => {
            let constr = constr
                .add(&Expression { ast: *cond.clone() }, &Truthy)
                .add(&Expression { ast: *then.clone() }, &AnyExpr);
            let (constr, env) = generate(cond, env, ctx, &constr)?;
            generate(then, &env, ctx, &constr)
        }

        Node::Case { .. } => Err(vec![TypeErr::new(&ast.pos, "Case cannot be top level")]),
        Node::Match { cond, cases } => {
            let mut constr_env = (constr.clone(), env.clone());
            let case_body = if let Some(case) = cases.first() {
                if env.state.expect_expr {
                    Some(Expression { ast: case.clone() })
                } else {
                    None
                }
            } else {
                None
            };

            for case in cases {
                match &case.node {
                    Node::Case { cond: case_cond, body } => {
                        constr_env.0 =
                            constr_env.0.add(&Expression { ast: *cond.clone() }, &Expression {
                                ast: *case_cond.clone()
                            });
                        if let Some(case_body) = &case_body {
                            constr_env.0 =
                                constr_env.0.add(case_body, &Expression { ast: *body.clone() });
                        }
                        constr_env = generate(body, &constr_env.1, ctx, &constr_env.0)?;
                    }
                    _ => return Err(vec![TypeErr::new(&case.pos, "Expected case")])
                }
            }

            generate(cond, &constr_env.1, ctx, &constr_env.0)
        }

        Node::For { expr, col, body } => {
            let constr = constr.add(&Expression { ast: *col.clone() }, &Collection {
                ty: Some(Box::from(Expression { ast: *expr.clone() }))
            });
            let (constr, env) = generate(expr, env, ctx, &constr)?;
            let (constr, env) = generate(col, &env, ctx, &constr)?;
            generate(body, &env, ctx, &constr)
        }
        Node::Step { amount } => {
            let type_name = TypeName::from(ty::concrete::INT_PRIMITIVE);
            let constr = constr.add(&Expression { ast: *amount.clone() }, &Type { type_name });
            Ok((constr, env.clone()))
        }
        Node::While { cond, body } => {
            let constr = constr.add(&Expression { ast: *cond.clone() }, &Truthy);
            generate(body, env, ctx, &constr)
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected control flow")])
    }
}
