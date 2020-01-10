use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::cons::Constraints;
use crate::type_checker::constraints::generate::{gen_vec, generate};
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::type_result::TypeErr;

pub fn gen_class(ast: &AST, env: &Environment, ctx: &Context, constr: &Constraints) -> Constrained {
    match &ast.node {
        Node::Class { body, .. } =>
            if let Some(body) = body {
                match &body.node {
                    Node::Block { statements } => gen_vec(statements, env, ctx, constr),
                    _ => Err(vec![TypeErr::new(&body.pos, "Expected code block")])
                }
            } else {
                Ok((constr.clone(), env.clone()))
            },

        Node::TypeDef { body, .. } =>
            if let Some(body) = body {
                generate(body, env, ctx, constr)
            } else {
                Ok((constr.clone(), env.clone()))
            },
        Node::TypeAlias { conditions, .. } => gen_vec(conditions, env, ctx, constr),
        Node::Condition { cond, _else } => {
            let (constr, env) = generate(cond, env, ctx, constr)?;
            if let Some(el) = _else {
                generate(el, &env, ctx, &constr)
            } else {
                Ok((constr, env))
            }
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected class or type definition")])
    }
}
