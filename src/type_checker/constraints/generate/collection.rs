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
        Node::Set { elements } | Node::List { elements } if elements.first().is_some() => {
            let first = elements.first().unwrap();
            let mut constr_env = (constr.clone(), env.clone());
            for element in elements {
                constr_env.0 = constr_env
                    .0
                    .add(&Expression { ast: element.clone() }, &Expression { ast: first.clone() });
                constr_env = generate(element, &env, &ctx, &constr)?;
            }
            Ok(constr_env)
        }
        Node::Set { .. } | Node::List { .. } => Ok((constr.clone(), env.clone())),
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
    constr: &Constraints
) -> Constrained<Constraints> {
    let exp_collection = Collection { ty: Some(Box::from(Expression { ast: lookup.clone() })) };
    Ok(constr.add(&Expression { ast: collection.clone() }, &exp_collection))
}
