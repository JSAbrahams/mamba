use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::constraint::expected::Expect::*;
use crate::type_checker::constraints::constraint::expected::Expected;
use crate::type_checker::constraints::constraint::Constraints;
use crate::type_checker::constraints::generate::generate;
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::type_result::TypeErr;

pub fn gen_stmt(ast: &AST, env: &Environment, ctx: &Context, constr: &Constraints) -> Constrained {
    match &ast.node {
        Node::Raise { error } => generate(error, env, ctx, constr),
        Node::Return { expr } => {
            let left = Expected::new(&expr.pos, &Expression { ast: *expr.clone() });
            let constr = constr.add(&left, &Expected::new(&expr.pos, &ExpressionAny));
            generate(expr, env, ctx, &constr)
        }
        Node::Print { expr } => {
            let left = Expected::new(&expr.pos, &Expression { ast: *expr.clone() });
            let constr = constr.add(&left, &Expected::new(&expr.pos, &ExpressionAny));
            generate(expr, env, ctx, &constr)
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected statement")])
    }
}
