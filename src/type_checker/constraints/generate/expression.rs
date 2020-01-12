use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::cons::Constraints;
use crate::type_checker::constraints::cons::Expect::{Expression, Nullable};
use crate::type_checker::constraints::generate::definition::constrain_args;
use crate::type_checker::constraints::generate::generate;
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::type_result::TypeErr;

pub fn gen_expr(ast: &AST, env: &Environment, ctx: &Context, constr: &Constraints) -> Constrained {
    match &ast.node {
        Node::AnonFun { args, body } => {
            let (constr, env) = constrain_args(args, env, ctx, constr)?;
            generate(body, &env, ctx, &constr)
        }
        Node::Id { lit } => {
            env.lookup_new(lit, &ast.pos)?;
            Ok((constr.clone(), env.clone()))
        }
        Node::Question { left, right } => {
            let nullable = Nullable { expect: Box::from(Expression { ast: *left.clone() }) };
            let constr = constr.add(&Expression { ast: *right.clone() }, &nullable);
            let (constr, env) = generate(left, env, ctx, &constr)?;
            generate(right, &env, ctx, &constr)
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected expression")])
    }
}
