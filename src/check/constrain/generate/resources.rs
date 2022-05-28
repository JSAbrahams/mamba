use std::convert::TryFrom;

use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::expected::Expect::{ExpressionAny, Raises, Type};
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::generate::{Constrained, generate};
use crate::check::constrain::generate::definition::identifier_from_var;
use crate::check::constrain::generate::env::Environment;
use crate::check::context::Context;
use crate::check::name::Name;
use crate::check::result::{TypeErr, TypeResult};
use crate::parse::ast::{AST, Node};

pub fn gen_resources(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    match &ast.node {
        Node::Raises { expr_or_stmt, errors } => {
            let mut constr = constr.clone();
            for error in errors {
                let exp = Expected::new(error.pos, &Raises { name: Name::try_from(error)? });
                constr = constrain_raises(&exp, &env.raises, &mut constr)?;
            }
            // raises expression has type of contained expression
            constr.add("raises", &Expected::try_from((ast, &env.var_mappings))?, &Expected::try_from((expr_or_stmt, &env.var_mappings))?);
            generate(expr_or_stmt, env, ctx, &mut constr)
        }
        Node::With { resource, alias: Some((alias, mutable, ty)), expr } => {
            constr.new_set(true);
            let resource_exp = Expected::try_from((resource, &env.var_mappings))?;
            constr.add("with as", &resource_exp, &Expected::try_from((alias, &env.var_mappings))?);
            constr.add("with as", &resource_exp, &Expected::new(resource.pos, &ExpressionAny));

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
                &Expected::new(resource.pos, &ExpressionAny),
            );
            let (mut constr, env) = generate(resource, env, ctx, constr)?;
            constr.exit_set(ast.pos)?;
            generate(expr, &env, ctx, &mut constr)
        }

        _ => Err(vec![TypeErr::new(ast.pos, "Expected resources")])
    }
}

/// Constrain expected to raises
///
/// This indicates that the type should be contained within the set of the
/// raises constraint. Does not constrain if top-level in constr builder,
/// meaning that we do not constrain raises in top-level scripts.
pub fn constrain_raises(
    raises: &Expected,
    env_raises: &Option<Expected>,
    constr: &mut ConstrBuilder,
) -> TypeResult<ConstrBuilder> {
    if constr.level == 0 {
        return Ok(constr.clone());
    }

    if let Some(env_raises) = env_raises {
        constr.add("raises", env_raises, raises);
        Ok(constr.clone())
    } else {
        Err(vec![TypeErr::new(raises.pos, "Unexpected raise")])
    }
}
