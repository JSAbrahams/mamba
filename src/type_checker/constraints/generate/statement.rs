use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::cons::Constraints;
use crate::type_checker::constraints::cons::Expect::{AnyExpr, Expression};
use crate::type_checker::constraints::generate::generate;
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::type_result::TypeErr;

pub fn gen_stmt(ast: &AST, env: &Environment, ctx: &Context, constr: &Constraints) -> Constrained {
    // TODO add constraints for checking that an exception is raised
    match &ast.node {
        Node::Raise { error } => generate(error, env, ctx, constr),
        Node::Return { expr } => {
            let constr = constr.add(&Expression { ast: *expr.clone() }, &AnyExpr);
            generate(expr, env, ctx, &constr)
        }
        Node::Print { expr } => {
            let constr = constr.add(&Expression { ast: *expr.clone() }, &AnyExpr);
            generate(expr, env, ctx, &constr)
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected statement")])
    }
}
