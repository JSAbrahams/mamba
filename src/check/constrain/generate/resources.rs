use std::convert::TryFrom;

use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::expected::Expect::Type;
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::generate::{Constrained, generate};
use crate::check::constrain::generate::definition::identifier_from_var;
use crate::check::constrain::generate::env::Environment;
use crate::check::context::Context;
use crate::check::name::Name;
use crate::check::result::TypeErr;
use crate::parse::ast::{AST, Node};

pub fn gen_resources(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    match &ast.node {
        Node::With { resource, alias: Some((alias, mutable, ty)), expr } => {
            constr.add("with alias", &Expected::from(resource), &Expected::from(alias), env);
            constr.add("with resource", &Expected::from(resource), &Expected::any(resource.pos), env);

            if let Some(ty) = ty {
                let ty_exp = Type { name: Name::try_from(ty)? };
                constr.add("with alias type", &Expected::from(resource), &Expected::new(ty.pos, &ty_exp), env);
            }

            let resource_env = generate(resource, &env.is_destruct_mode(true), ctx, constr)?
                .is_destruct_mode(false);

            constr.branch_point();
            let ty = if let Some(ty) = ty { Some(Name::try_from(ty)?) } else { None };
            let resource_env = identifier_from_var(
                alias,
                &ty,
                &Some(alias.clone()),
                *mutable,
                ctx,
                constr,
                &resource_env.is_def_mode(true),
            )?;

            generate(expr, &resource_env.is_def_mode(false), ctx, constr)?;
            Ok(env.clone())
        }
        Node::With { resource, expr, .. } => {
            constr.add("with", &Expected::from(resource), &Expected::any(resource.pos), env);

            let resource_env = generate(resource, env, ctx, constr)?;
            generate(expr, &resource_env, ctx, constr)?;
            Ok(env.clone())
        }

        _ => Err(vec![TypeErr::new(ast.pos, "Expected resources")])
    }
}
