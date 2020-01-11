use std::convert::TryFrom;

use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::cons::Constraints;
use crate::type_checker::constraints::cons::Expect::Type;
use crate::type_checker::constraints::generate::definition::constrain_args;
use crate::type_checker::constraints::generate::{gen_vec, generate};
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::type_name::TypeName;
use crate::type_checker::type_result::TypeErr;
use std::ops::Deref;

pub fn gen_class(ast: &AST, env: &Environment, ctx: &Context, constr: &Constraints) -> Constrained {
    match &ast.node {
        Node::Class { body: Some(body), args, ty, .. } => match &body.node {
            Node::Block { statements } => {
                let (constr, env) = constrain_args(args, env, ctx, constr)?;
                let type_name = TypeName::try_from(ty.deref())?;
                let env = env.in_class_new(&Type { type_name });
                gen_vec(statements, &env, ctx, &constr)
            }
            _ => Err(vec![TypeErr::new(&body.pos, "Expected code block")])
        },
        Node::Class { .. } => Ok((constr.clone(), env.clone())),

        Node::TypeDef { body: Some(body), ty, .. } => {
            let type_name = TypeName::try_from(ty.deref())?;
            let env = env.in_class_new(&Type { type_name });
            generate(body, &env, ctx, constr)
        }
        Node::TypeDef { .. } => Ok((constr.clone(), env.clone())),

        Node::TypeAlias { conditions, ty, .. } => {
            let type_name = TypeName::try_from(ty.deref())?;
            let env = env.in_class_new(&Type { type_name });
            gen_vec(conditions, &env, ctx, constr)
        }
        Node::Condition { cond, el: Some(el) } => {
            let (constr, env) = generate(cond, env, ctx, constr)?;
            generate(el, &env, ctx, &constr)
        }
        Node::Condition { cond, .. } => generate(cond, env, ctx, constr),

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected class or type definition")])
    }
}
