use std::convert::TryFrom;

use crate::check::checker_result::TypeErr;
use crate::check::constraints::constraint::builder::ConstrBuilder;
use crate::check::constraints::constraint::expected::Expect::*;
use crate::check::constraints::constraint::expected::{Expect, Expected};
use crate::check::constraints::generate::gen_vec;
use crate::check::constraints::Constrained;
use crate::check::context::Context;
use crate::check::environment::name::Identifier;
use crate::check::environment::Environment;
use crate::parse::ast::{Node, AST};

pub fn gen_coll(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder
) -> Constrained {
    match &ast.node {
        Node::Set { elements } | Node::List { elements } | Node::Tuple { elements } =>
            gen_vec(elements, env, ctx, constr),

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
pub fn constr_col(collection: &AST, constr: &mut ConstrBuilder) -> ConstrBuilder {
    let col = match &collection.node {
        Node::Set { elements } | Node::List { elements } | Node::Tuple { elements } =>
            if let Some(first) = elements.first() {
                for element in elements {
                    constr.add(&Expected::from(element), &Expected::from(first))
                }
                Expect::Collection {
                    ty: Box::from(Expected::new(&first.pos, &Expression { ast: first.clone() }))
                }
            } else {
                Expect::Collection { ty: Box::from(Expected::new(&collection.pos, &ExpressionAny)) }
            },

        _ => Expect::Collection { ty: Box::from(Expected::new(&collection.pos, &ExpressionAny)) }
    };

    let col_exp = Expected::new(&collection.pos, &col);
    constr.add(&Expected::from(collection), &col_exp);
    constr.clone()
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
    constr: &mut ConstrBuilder
) -> Constrained {
    let identifier = Identifier::try_from(lookup)?;
    let mut env = env.clone();
    let any = Expected::new(&lookup.pos, &ExpressionAny);
    for (mutable, var) in identifier.fields() {
        env = env.insert_var(mutable, &var, &any);
    }

    let exp_collection = Collection { ty: Box::from(Expected::from(lookup)) };
    constr.add(&Expected::new(&lookup.pos, &exp_collection), &Expected::from(col));
    Ok((constr.clone(), env.clone()))
}
