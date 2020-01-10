use std::ops::Deref;

use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::cons::{Constraints, Expect};
use crate::type_checker::constraints::generate::generate;
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::type_result::TypeErr;

pub fn gen_resources(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &Constraints
) -> Constrained {
    match &ast.node {
        Node::Raises { .. } => unimplemented!(),
        Node::With { resource, alias, expr } => {
            let constr = if let Some(alias) = alias {
                constr.add(
                    &Expect::Expression { ast: resource.deref().clone() },
                    &Expect::Expression { ast: alias.deref().clone() }
                )
            } else {
                constr.clone()
            };

            let (constr, env) = generate(resource, env, ctx, &constr)?;
            let (constr, env) = if let Some(alias) = alias {
                generate(alias, &env, ctx, &constr)
            } else {
                (constr, env)
            };
            generate(expr, &env, ctx, &constr)
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected resources")])
    }
}
