use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::constraint::Constraint;
use crate::check::constrain::generate::{generate, Constrained};
use crate::check::constrain::generate::resources::constrain_raises;
use crate::check::context::Context;
use crate::check::constrain::generate::env::Environment;
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
            let mut constr = constrain_raises(&Expected::try_from((error, &env.var_mappings))?, &env.raises, constr)?;
            generate(error, env, ctx, &mut constr)
        }
        Node::ReturnEmpty => Ok((constr.clone(), env.clone())),
        Node::Return { expr } =>
            if let Some(expected_ret_ty) = &env.return_type {
                constr.add("return", &expected_ret_ty, &Expected::try_from((expr, &env.var_mappings))?);
                generate(expr, env, ctx, constr)
            } else {
                Err(vec![TypeErr::new(&ast.pos, "Return outside function with return type")])
            },
        Node::Print { expr } => {
            let con = Constraint::stringy("print", &Expected::try_from((expr, &env.var_mappings))?);
            constr.add_constr(&con);
            generate(expr, env, ctx, constr)
        }
        _ => Err(vec![TypeErr::new(&ast.pos, "Expected statement")])
    }
}
