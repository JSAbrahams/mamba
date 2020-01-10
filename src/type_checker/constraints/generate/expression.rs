use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::cons::Constraints;
use crate::type_checker::constraints::cons::Expect::{AnyExpr, Expression, Nullable};
use crate::type_checker::constraints::generate::generate;
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::type_result::TypeErr;

pub fn gen_expr(ast: &AST, env: &Environment, ctx: &Context, constr: &Constraints) -> Constrained {
    match &ast.node {
        Node::AnonFun { .. } => unimplemented!(),
        Node::Id { lit } if env.lookup_new(lit, &ast.pos)? => Ok((constr.clone(), env.clone())),
        Node::Id { lit } =>
            Err(vec![TypeErr::new(&ast.pos, &format!("Unknown variable: {}", lit))]),
        Node::Question { left, right } => {
            let constr = constr
                .add(&Expression { ast: *left.clone() }, &Nullable { expect: Box::from(AnyExpr) })
                .add(&Expression { ast: *right.clone() }, &AnyExpr);
            let (constr, env) = generate(left, env, ctx, &constr)?;
            generate(right, &env, ctx, &constr)
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected expression")])
    }
}
