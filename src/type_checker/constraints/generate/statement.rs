use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::cons::{Constraints, Expect};
use crate::type_checker::constraints::generate::generate;
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::type_result::TypeErr;
use std::ops::Deref;

pub fn gen_statement(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &Constraints
) -> Constrained {
    match &ast.node {
        Node::Raise { .. } => unimplemented!(),
        Node::Return { expr } => {
            let constr = constr
                .add(&Expect::Expression { ast: expr.deref().clone() }, &Expect::AnyExpression);
            generate(expr, env, ctx, &constr)
        }
        Node::Print { expr } => {
            let constr = constr
                .add(&Expect::Expression { ast: expr.deref().clone() }, &Expect::AnyExpression);
            generate(expr, env, ctx, &constr)
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected statement")])
    }
}
