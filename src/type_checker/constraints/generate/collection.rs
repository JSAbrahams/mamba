use std::convert::TryFrom;

use crate::parser::ast::{Node, AST};
use crate::type_checker::checker_result::TypeErr;
use crate::type_checker::constraints::constraint::builder::ConstrBuilder;
use crate::type_checker::constraints::constraint::expected::Expect::*;
use crate::type_checker::constraints::constraint::expected::Expected;
use crate::type_checker::constraints::generate::{gen_vec, generate};
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::Context;
use crate::type_checker::environment::name::Identifier;
use crate::type_checker::environment::Environment;

pub fn gen_coll(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &ConstrBuilder
) -> Constrained {
    match &ast.node {
        Node::Set { elements } | Node::List { elements } =>
            if let Some(first) = elements.first() {
                let mut res = (constr.clone(), env.clone());
                let first_exp = Expression { ast: first.clone() };
                for element in elements {
                    let left = Expected::from(element);
                    res.0.add(&left, &Expected::new(&first.pos, &first_exp));
                    res = generate(element, &res.1, &ctx, &mut res.0)?;
                }
                Ok(res)
            } else {
                Ok((constr.clone(), env.clone()))
            },
        Node::Tuple { elements } => gen_vec(elements, env, ctx, constr),

        Node::SetBuilder { .. } =>
            Err(vec![TypeErr::new(&ast.pos, "Set builders currently not supported")]),
        Node::ListBuilder { .. } =>
            Err(vec![TypeErr::new(&ast.pos, "List builders currently not supported")]),
        _ => Err(vec![TypeErr::new(&ast.pos, "Expected collection")])
    }
}

pub fn constrain_collection(
    collection: &AST,
    lookup: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder
) -> Constrained {
    let identifier = Identifier::try_from(lookup)?;
    let mut env = env.clone();
    for (mutable, var) in identifier.fields() {
        env = env.insert_new(mutable, &var, &ExpressionAny);
    }

    let exp_collection = Collection { ty: Box::from(Expression { ast: lookup.clone() }) };
    let left = Expected::from(collection);
    constr.add(&left, &Expected::new(&lookup.pos, &exp_collection));
    generate(collection, &env, ctx, constr)
}
