use crate::check::checker_result::TypeErr;
use crate::check::constraints::constraint::builder::ConstrBuilder;
use crate::check::constraints::constraint::expected::Expect::*;
use crate::check::constraints::constraint::expected::Expected;
use crate::check::constraints::generate::generate;
use crate::check::constraints::generate::resources::constrain_raises;
use crate::check::constraints::Constrained;
use crate::check::context::Context;
use crate::check::environment::Environment;
use crate::parse::ast::{Node, AST};

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
        Node::ReturnEmpty => Ok((constr.clone(), env.clone())),
        Node::Return { expr } =>
            if let Some(expected_ret_ty) = &env.return_type {
                let left = Expected::from(expr);
                constr.add(&left, &expected_ret_ty);
                generate(expr, env, ctx, constr)
            } else {
                Err(vec![TypeErr::new(&ast.pos, "Return outside function with return type")])
            },
        Node::Print { expr } => {
            let left = Expected::from(expr);
            constr.add(&left, &Expected::new(&expr.pos, &Stringy));
            generate(expr, env, ctx, constr)
        }
        _ => Err(vec![TypeErr::new(&ast.pos, "Expected statement")])
    }
}
