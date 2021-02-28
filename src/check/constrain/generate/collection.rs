use std::convert::TryFrom;

use crate::check::constrain::Constrained;
use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::expected::{Expect, Expected};
use crate::check::constrain::constraint::expected::Expect::*;
use crate::check::constrain::generate::gen_vec;
use crate::check::context::Context;
use crate::check::env::Environment;
use crate::check::ident::Identifier;
use crate::check::result::{TypeErr, TypeResult};
use crate::parse::ast::{AST, Node};

pub fn gen_coll(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    match &ast.node {
        Node::Set { elements } | Node::List { elements } | Node::Tuple { elements } =>
            gen_vec(elements, env, ctx, &constr_col(ast, constr)?),

        Node::SetBuilder { .. } =>
            Err(vec![TypeErr::new(&ast.pos, "Set builders currently not supported")]),
        Node::ListBuilder { .. } =>
            Err(vec![TypeErr::new(&ast.pos, "List builders currently not supported")]),
        _ => Err(vec![TypeErr::new(&ast.pos, "Expected collection")])
    }
}

/// Generate constraint for collection by taking first element
///
/// The assumption here being that every element in the set has the same type.
pub fn constr_col(collection: &AST, constr: &mut ConstrBuilder) -> TypeResult<ConstrBuilder> {
    let (msg, col) = match &collection.node {
        Node::Set { elements } | Node::List { elements } => {
            let ty = if let Some(first) = elements.first() {
                for element in elements {
                    constr.add(
                        "collection item",
                        &Expected::try_from(first)?,
                        &Expected::try_from(element)?,
                    )
                }
                Box::from(Expected::new(&collection.pos, &Collection {
                    ty: Box::from(Expected::new(&first.pos, &Expression { ast: first.clone() }))
                }))
            } else {
                Box::from(Expected::new(&collection.pos, &Collection {
                    ty: Box::from(Expected::new(&collection.pos, &ExpressionAny))
                }))
            };

            ("collection", Expect::Collection { ty })
        }
        Node::Tuple { elements } => {
            let elements =
                elements.iter().map(|ast| Expected::try_from(ast)).collect::<Result<_, _>>()?;
            ("tuple", Expect::Tuple { elements })
        }

        _ => ("collection", Expect::Collection {
            ty: Box::from(Expected::new(&collection.pos, &ExpressionAny))
        })
    };

    let col_exp = Expected::new(&collection.pos, &col);
    constr.add(msg, &col_exp, &Expected::try_from(collection)?);
    Ok(constr.clone())
}

/// Constrain lookup an collection.
///
/// This is done by constraining the given expected collection with an expected
/// collection with the lookup parameter. Therefore, the type of the lookup, and
/// the type of the given collection's parameter must be the same.
pub fn gen_collection_lookup(
    lookup: &AST,
    col: &AST,
    env: &Environment,
    constr: &mut ConstrBuilder,
) -> Constrained {
    let (mut env, mut vars) = (env.clone(), vec![]);
    for (mutable, var) in Identifier::try_from(lookup)?.fields() {
        vars.push(var.clone());
        env = env.insert_var(mutable, &var, &Expected::new(&lookup.pos, &ExpressionAny));
    }

    constr.add_with_identifier(
        "collection lookup",
        &Expected::new(&lookup.pos, &Collection { ty: Box::from(Expected::try_from(lookup)?) }),
        &Expected::try_from(col)?,
        &vars,
    );

    Ok((constr.clone(), env))
}
