use std::convert::TryFrom;

use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::constraint::expected::Expect::*;
use crate::type_checker::constraints::constraint::expected::Expected;
use crate::type_checker::constraints::constraint::Constraints;
use crate::type_checker::constraints::generate::definition::identifier_from_var;
use crate::type_checker::constraints::generate::generate;
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::type_name::TypeName;
use crate::type_checker::type_result::TypeErr;

pub fn gen_resources(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &Constraints
) -> Constrained {
    match &ast.node {
        Node::Raises { expr_or_stmt, errors } => {
            let (constr, env) = constrain_raises(expr_or_stmt, errors, env, ctx, constr)?;
            generate(expr_or_stmt, &env, ctx, &constr)
        }
        Node::With { resource, alias: Some((alias, mutable, ty)), expr } => {
            let left = Expected::new(&resource.pos, &Expression { ast: *resource.clone() });
            let constr =
                constr.add(&left, &Expected::new(&alias.pos, &Expression { ast: *alias.clone() }));

            let (constr, env) = identifier_from_var(alias, ty, *mutable, &constr, env)?;
            let (constr, env) = generate(resource, &env, ctx, &constr)?;
            generate(expr, &env, ctx, &constr)
        }
        Node::With { resource, expr, .. } => {
            let (constr, env) = generate(resource, env, ctx, &constr)?;
            generate(expr, &env, ctx, &constr)
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected resources")])
    }
}

pub fn constrain_raises(
    expr: &AST,
    errors: &Vec<AST>,
    env: &Environment,
    ctx: &Context,
    constr: &Constraints
) -> Constrained {
    let mut res = (constr.clone(), env.clone());
    for error in errors {
        let type_name = TypeName::try_from(error)?;
        let left = Expected::new(&expr.pos, &Expression { ast: expr.clone() });
        res.0 = res.0.add(&left, &Expected::new(&error.pos, &Raises { type_name }));
        res = generate(error, &res.1, ctx, &res.0)?;
    }

    Ok(res)
}
