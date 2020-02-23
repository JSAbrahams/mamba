use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::expected::Expect::*;
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::generate::generate;
use crate::check::constrain::generate::resources::constrain_raises;
use crate::check::constrain::Constrained;
use crate::check::context::Context;
use crate::check::env::Environment;
use crate::check::result::TypeErr;
use crate::parse::ast::{Node, AST};
use std::convert::TryFrom;

pub fn gen_stmt(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder
) -> Constrained {
    match &ast.node {
        Node::Raise { error } => {
            let mut constr = constrain_raises(&Expected::try_from(error)?, &env.raises, constr)?;
            generate(error, env, ctx, &mut constr)
        }
        Node::ReturnEmpty => Ok((constr.clone(), env.clone())),
        Node::Return { expr } =>
            if let Some(expected_ret_ty) = &env.return_type {
                constr.add(&expected_ret_ty, &Expected::try_from(expr)?);
                generate(expr, env, ctx, constr)
            } else {
                Err(vec![TypeErr::new(&ast.pos, "Return outside function with return type")])
            },
        Node::Print { expr } => {
            let left = Expected::try_from(expr)?;
            constr.add(&left, &Expected::new(&expr.pos, &Stringy));
            generate(expr, env, ctx, constr)
        }
        _ => Err(vec![TypeErr::new(&ast.pos, "Expected statement")])
    }
}
