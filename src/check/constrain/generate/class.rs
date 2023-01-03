use std::convert::TryFrom;

use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::generate::{Constrained, gen_vec, generate};
use crate::check::constrain::generate::env::Environment;
use crate::check::context::Context;
use crate::check::name::string_name::StringName;
use crate::check::result::TypeErr;
use crate::parse::ast::{AST, Node};

pub fn gen_class(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    match &ast.node {
        Node::Class { body: Some(body), ty, .. } | Node::TypeDef { body: Some(body), ty, .. } =>
            match &body.node {
                Node::Block { statements } => constrain_class_body(statements, ty, env, ctx, constr),
                _ => Err(vec![TypeErr::new(body.pos, "Expected code block")])
            },
        Node::Class { .. } | Node::TypeDef { .. } => Ok(env.clone()),

        Node::TypeAlias { conditions, isa, .. } => constrain_class_body(conditions, isa, env, ctx, constr),
        Node::Condition { cond, el: Some(el) } => {
            generate(cond, env, ctx, constr)?;
            generate(el, env, ctx, constr)
        }
        Node::Condition { cond, .. } => generate(cond, env, ctx, constr),

        _ => Err(vec![TypeErr::new(ast.pos, "Expected class or type definition")])
    }
}

pub fn constrain_class_body(
    statements: &[AST],
    ty: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    let class_lvl = constr.new_set();

    let name = StringName::try_from(ty)?;
    let env_with_class_fields = env.in_class(true, &name, ty.pos);
    gen_vec(statements, &env_with_class_fields, true, ctx, constr)?;

    constr.exit_set_to(class_lvl, ty.pos)?;
    Ok(env_with_class_fields.clone())
}
