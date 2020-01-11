use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::cons::Constraints;
use crate::type_checker::constraints::generate::definition::constrain_args;
use crate::type_checker::constraints::generate::{gen_vec, generate};
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::type_result::TypeErr;

pub fn gen_class(ast: &AST, env: &Environment, ctx: &Context, constr: &Constraints) -> Constrained {
    match &ast.node {
        Node::Class { body: Some(body), args, .. } => match &body.node {
            Node::Block { statements } => {
                let (constr, env) = constrain_args(args, env, ctx, constr)?;
                gen_vec(statements, &env, ctx, &constr)
            }
            _ => Err(vec![TypeErr::new(&body.pos, "Expected code block")])
        },
        Node::Class { .. } => Ok((constr.clone(), env.clone())),

        Node::TypeDef { body: Some(body), .. } => generate(body, env, ctx, constr),
        Node::TypeDef { .. } => Ok((constr.clone(), env.clone())),

        Node::TypeAlias { conditions, .. } => gen_vec(conditions, env, ctx, constr),
        Node::Condition { cond, el: Some(el) } => {
            let (constr, env) = generate(cond, env, ctx, constr)?;
            generate(el, &env, ctx, &constr)
        }
        Node::Condition { cond, .. } => generate(cond, env, ctx, constr),

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected class or type definition")])
    }
}
