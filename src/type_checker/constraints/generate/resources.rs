use std::convert::TryFrom;

use crate::parser::ast::{Node, AST};
use crate::type_checker::checker_result::{TypeErr, TypeResult};
use crate::type_checker::constraints::constraint::builder::ConstrBuilder;
use crate::type_checker::constraints::constraint::expected::Expect::{ExpressionAny, Type};
use crate::type_checker::constraints::constraint::expected::Expected;
use crate::type_checker::constraints::generate::definition::identifier_from_var;
use crate::type_checker::constraints::generate::generate;
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::ty_name::TypeName;

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
                let exp = Expected::new(&error.pos, &Type { type_name: TypeName::try_from(ast)? });
                constr = constrain_raises(&exp, &env.raises, &mut constr)?;
            }
            // raises expression has type of contained expression
            constr.add(&Expected::from(ast), &Expected::from(expr_or_stmt));
            generate(expr_or_stmt, &env, ctx, &mut constr)
        }
        Node::With { resource, alias: Some((alias, mutable, ty)), expr } => {
            constr.new_set(true);
            let resource_exp = Expected::from(resource);
            constr.add(&resource_exp, &Expected::from(alias));
            constr.add(&resource_exp, &Expected::new(&resource.pos, &ExpressionAny));

            if let Some(ty) = ty {
                let ty_exp = Type { type_name: TypeName::try_from(ty)? };
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
            constr.add(&Expected::from(resource), &Expected::new(&resource.pos, &ExpressionAny));
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
