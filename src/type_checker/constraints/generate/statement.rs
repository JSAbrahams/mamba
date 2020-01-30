use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::constraint::builder::ConstrBuilder;
use crate::type_checker::constraints::constraint::expected::Expect::*;
use crate::type_checker::constraints::constraint::expected::Expected;
use crate::type_checker::constraints::generate::generate;
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::type_result::TypeErr;

pub fn gen_stmt(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder
) -> Constrained {
    match &ast.node {
        Node::Raise { error } => generate(error, env, ctx, constr),
        Node::Return { expr } =>
            if let Some(expected_ret_ty) = &env.state.ret_ty {
                let left = Expected::new(&expr.pos, &Expression { ast: *expr.clone() });
                constr.add(&left, &expected_ret_ty);
                generate(expr, env, ctx, constr)
            } else {
                Err(vec![TypeErr::new(&ast.pos, "Return statement only possible in function body")])
            },
        Node::Print { expr } => {
            let left = Expected::from(expr);
            constr.add(&left, &Expected::new(&expr.pos, &ExpressionAny));
            generate(expr, env, ctx, constr)
        }
        _ => Err(vec![TypeErr::new(&ast.pos, "Expected statement")])
    }
}
