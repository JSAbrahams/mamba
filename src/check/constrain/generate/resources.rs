use std::convert::TryFrom;

use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::expected::{Expect, Expected};
use crate::check::constrain::constraint::expected::Expect::Type;
use crate::check::constrain::generate::{Constrained, generate};
use crate::check::constrain::generate::definition::identifier_from_var;
use crate::check::constrain::generate::env::Environment;
use crate::check::context::Context;
use crate::check::name::{Any, Name};
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
            constr.new_set(true);
            let resource_exp = Expected::try_from((resource, &env.var_mappings))?;
            constr.add("with as", &resource_exp, &Expected::try_from((alias, &env.var_mappings))?);
            constr.add("with as", &resource_exp, &Expected::new(resource.pos, &Expect::any()));

            if let Some(ty) = ty {
                let ty_exp = Type { name: Name::try_from(ty)? };
                constr.add("with as", &resource_exp, &Expected::new(ty.pos, &ty_exp));
            }

            let (mut constr, env) = generate(resource, env, ctx, constr)?;

            constr.new_set(true);
            constr.remove_expected(&resource_exp);
            let (mut constr, env) = identifier_from_var(
                alias,
                ty,
                &Some(alias.clone()),
                *mutable,
                ctx,
                &mut constr,
                &env.define_mode(true),
            )?;
            let (mut constr, env) = generate(expr, &env, ctx, &mut constr)?;
            constr.exit_set(ast.pos)?;

            constr.exit_set(ast.pos)?;
            Ok((constr, env))
        }
        Node::With { resource, expr, .. } => {
            constr.new_set(true);
            constr.add(
                "with",
                &Expected::try_from((resource, &env.var_mappings))?,
                &Expected::new(resource.pos, &Expect::any()),
            );
            let (mut constr, env) = generate(resource, env, ctx, constr)?;
            constr.exit_set(ast.pos)?;
            generate(expr, &env, ctx, &mut constr)
        }

        _ => Err(vec![TypeErr::new(ast.pos, "Expected resources")])
    }
}
