use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::cons::{Constraints, Expect};
use crate::type_checker::constraints::generate::generate;
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::type_result::TypeErr;
use std::ops::Deref;

pub fn gen_expression(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &Constraints
) -> Constrained {
    match &ast.node {
        Node::AnonFun { .. } => unimplemented!(),
        Node::Id { lit } =>
            if env.lookup_new(lit, &ast.pos).is_ok() {
                Ok((constr.clone(), env.clone()))
            } else {
                Err(vec![TypeErr::new(&ast.pos, &format!("Unknown variable: {}", lit))])
            },

        Node::Question { left, right } => {
            let constr = constr
                .add(&Expect::Expression { ast: left.deref().clone() }, &Expect::Nullable {
                    expect: Box::from(Expect::AnyExpression)
                });
            let constr = constr
                .add(&Expect::Expression { ast: right.deref().clone() }, &Expect::AnyExpression);
            let (constr, env) = generate(left, env, ctx, &constr)?;
            generate(right, &env, ctx, &constr)
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected expression")])
    }
}
