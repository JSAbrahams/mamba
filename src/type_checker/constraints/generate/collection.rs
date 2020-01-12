use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::cons::Constraints;
use crate::type_checker::constraints::cons::Expect::{Collection, Expression};
use crate::type_checker::constraints::generate::{gen_vec, generate};
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::type_result::TypeErr;

pub fn gen_coll(ast: &AST, env: &Environment, ctx: &Context, constr: &Constraints) -> Constrained {
    match &ast.node {
        Node::Set { elements } | Node::List { elements } =>
            if let Some(first) = elements.first() {
                let mut res = (constr.clone(), env.clone());
                let first_exp = Expression { ast: first.clone() };
                for element in elements {
                    let element_expr = Expression { ast: element.clone() };
                    res.0 = res.0.add(&element_expr, &first_exp);
                    res = generate(element, &res.1, &ctx, &res.0)?;
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
    constr: &Constraints
) -> Constrained {
    let exp_collection = Collection { ty: Box::from(Expression { ast: lookup.clone() }) };
    let constr = constr.add(&Expression { ast: collection.clone() }, &exp_collection);
    let (constr, env) = generate(lookup, env, ctx, &constr)?;
    generate(collection, &env, ctx, &constr)
}
