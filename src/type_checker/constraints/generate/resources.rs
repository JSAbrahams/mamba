use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::cons::Constraints;
use crate::type_checker::constraints::cons::Expect::Expression;
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
        Node::With { resource, alias: Some(alias), expr } => {
            let constr = constr
                .add(&Expression { ast: *resource.clone() }, &Expression { ast: *alias.clone() });
            let (constr, env) = generate(resource, env, ctx, &constr)?;
            let (constr, env) = generate(alias, &env, ctx, &constr)?;
            generate(expr, &env, ctx, &constr)
        }
        Node::With { resource, expr, .. } => {
            let (constr, env) = generate(resource, env, ctx, &constr)?;
            generate(expr, &env, ctx, &constr)
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected resources")])
    }
}
