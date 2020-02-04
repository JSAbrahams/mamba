use crate::parser::ast::{Node, AST};
use crate::type_checker::checker_result::TypeErr;
use crate::type_checker::constraints::constraint::builder::ConstrBuilder;
use crate::type_checker::constraints::constraint::expected::Expect::*;
use crate::type_checker::constraints::constraint::expected::Expected;
use crate::type_checker::constraints::generate::generate;
use crate::type_checker::constraints::generate::resources::constrain_raises;
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;

pub fn gen_stmt(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder
) -> Constrained {
    match &ast.node {
        Node::Raise { error } => {
            let mut constr = constrain_raises(&Expected::from(error), &env.raises, constr)?;
            generate(error, env, ctx, &mut constr)
        }
        Node::Return { expr } =>
            if let Some(expected_ret_ty) = &env.return_type {
                let left = Expected::from(expr);
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
