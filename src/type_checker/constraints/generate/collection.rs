use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::cons::{Constraints, Expect};
use crate::type_checker::constraints::generate::generate;
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::type_result::TypeErr;

pub fn gen_collection(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &Constraints
) -> Constrained {
    match &ast.node {
        Node::SetBuilder { .. } => unimplemented!(),
        Node::ListBuilder { .. } => unimplemented!(),
        Node::Set { elements } | Node::List { elements } =>
            if let Some(first) = elements.first() {
                let mut constr_env = (constr.clone(), env.clone());
                for element in elements {
                    constr_env.0 = constr_env
                        .0
                        .add(&Expect::Expression { ast: element.clone() }, &Expect::Expression {
                            ast: first.clone()
                        });
                    constr_env = generate(element, &env, &ctx, &constr)?;
                }
                Ok(constr_env)
            } else {
                Ok((constr.clone(), env.clone()))
            },
        Node::Tuple { elements } => {
            let mut constr_env = (constr.clone(), env.clone());
            for element in elements {
                constr_env = generate(element, &env, &ctx, &constr)?;
            }
            Ok(constr_env)
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected collection")])
    }
}
