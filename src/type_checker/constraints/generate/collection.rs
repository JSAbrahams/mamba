use std::convert::TryFrom;

use crate::parser::ast::{Node, AST};
use crate::type_checker::checker_result::TypeErr;
use crate::type_checker::constraints::constraint::builder::ConstrBuilder;
use crate::type_checker::constraints::constraint::expected::Expect::*;
use crate::type_checker::constraints::constraint::expected::{Expect, Expected};
use crate::type_checker::constraints::generate::gen_vec;
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::Context;
use crate::type_checker::environment::name::Identifier;
use crate::type_checker::environment::Environment;

pub fn gen_coll(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder
) -> Constrained {
    match &ast.node {
        Node::Set { elements } | Node::List { elements } | Node::Tuple { elements } => {
            let (mut constr, _) = gen_collection(ast, constr);
            gen_vec(elements, env, ctx, &mut constr)
        }

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
pub fn gen_collection(collection: &AST, constr: &mut ConstrBuilder) -> (ConstrBuilder, Expected) {
    let col = match &collection.node {
        Node::Set { elements } | Node::List { elements } | Node::Tuple { elements } =>
            if let Some(first) = elements.first() {
                for element in elements {
                    constr.add(&Expected::from(element), &Expected::from(first))
                }
                Expect::Collection { ty: Box::from(Expression { ast: first.clone() }) }
            } else {
                Expect::Collection { ty: Box::from(ExpressionAny) }
            },

        _ => Expect::Collection { ty: Box::from(ExpressionAny) }
    };

    let col_exp = Expected::new(&collection.pos, &col);
    constr.add(&Expected::from(collection), &col_exp);
    (constr.clone(), col_exp.clone())
}

/// Constrain lookup an collection.
///
/// This is done by constraining the given expected collection with an expected
/// collection with the lookup parameter. Therefore, the type of the lookup, and
/// the type of the given collection's parameter must be the same.
pub fn gen_collection_lookup(
    lookup: &AST,
    col: &Expected,
    env: &Environment,
    constr: &mut ConstrBuilder
) -> Constrained {
    let identifier = Identifier::try_from(lookup)?;
    let mut env = env.clone();
    for (mutable, var) in identifier.fields() {
        env = env.insert_var(mutable, &var, &ExpressionAny);
    }

    let exp_collection = Collection { ty: Box::from(Expression { ast: lookup.clone() }) };
    constr.add(&Expected::new(&lookup.pos, &exp_collection), &col);
    Ok((constr.clone(), env.clone()))
}
