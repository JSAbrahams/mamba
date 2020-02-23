use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::expected::Expect::{ExpressionAny, Type};
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::generate::definition::identifier_from_var;
use crate::check::constrain::generate::generate;
use crate::check::constrain::Constrained;
use crate::check::context::name::NameUnion;
use crate::check::context::Context;
use crate::check::env::Environment;
use crate::check::result::{TypeErr, TypeResult};
use crate::parse::ast::{Node, AST};
use std::convert::TryFrom;

pub fn gen_resources(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder
) -> Constrained {
    match &ast.node {
        Node::Raises { expr_or_stmt, errors } => {
            let mut constr = constr.clone();
            for error in errors {
                let exp = Expected::new(&error.pos, &Type { name: NameUnion::try_from(error)? });
                constr = constrain_raises(&exp, &env.raises, &mut constr)?;
            }
            // raises expression has type of contained expression
            constr.add(&Expected::try_from(ast)?, &Expected::try_from(expr_or_stmt)?);
            generate(expr_or_stmt, &env, ctx, &mut constr)
        }
        Node::With { resource, alias: Some((alias, mutable, ty)), expr } => {
            constr.new_set(true);
            let resource_exp = Expected::try_from(resource)?;
            constr.add(&resource_exp, &Expected::try_from(alias)?);
            constr.add(&resource_exp, &Expected::new(&resource.pos, &ExpressionAny));

            if let Some(ty) = ty {
                let ty_exp = Type { name: NameUnion::try_from(ty)? };
                constr.add(&resource_exp, &Expected::new(&ty.pos, &ty_exp));
            }

            let (mut constr, env) = generate(resource, &env, ctx, constr)?;

            constr.new_set(true);
            constr.remove_expected(&resource_exp);
            let (mut constr, env) = identifier_from_var(
                alias,
                ty,
                &Some(alias.clone()),
                *mutable,
                ctx,
                &mut constr,
                &env
            )?;
            let (mut constr, env) = generate(expr, &env, ctx, &mut constr)?;
            constr.exit_set(&ast.pos)?;

            constr.exit_set(&ast.pos)?;
            Ok((constr, env))
        }
        Node::With { resource, expr, .. } => {
            constr.new_set(true);
            constr
                .add(&Expected::try_from(resource)?, &Expected::new(&resource.pos, &ExpressionAny));
            let (mut constr, env) = generate(resource, env, ctx, constr)?;
            constr.exit_set(&ast.pos)?;
            generate(expr, &env, ctx, &mut constr)
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected resources")])
    }
}

/// Constrain expected to raises
///
/// This indicates that the type should be contained within the set of the
/// raises constraint. Does not constrain if top-level in constr builder,
/// meaning that we do not constrain raises in top-level scripts.
pub fn constrain_raises(
    exp: &Expected,
    raises: &Option<Expected>,
    constr: &mut ConstrBuilder
) -> TypeResult<ConstrBuilder> {
    if constr.level == 0 {
        return Ok(constr.clone());
    }

    if let Some(raises) = raises {
        constr.add(&exp, raises);
        Ok(constr.clone())
    } else {
        Err(vec![TypeErr::new(&exp.pos, "Unexpected raise")])
    }
}
