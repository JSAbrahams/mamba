use std::convert::TryFrom;

use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::generate::{Constrained, gen_vec, generate};
use crate::check::constrain::generate::definition::id_from_var;
use crate::check::constrain::generate::env::Environment;
use crate::check::context::arg::python::SELF;
use crate::check::context::Context;
use crate::check::name::Name;
use crate::check::name::string_name::StringName;
use crate::check::result::TypeErr;
use crate::parse::ast::{AST, Node};
use crate::parse::ast::Node::Id;

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

        Node::TypeAlias { conditions, isa, ty } => {
            // Self is defined top level in type alias
            let var = AST::new(ty.pos, Id { lit: String::from(SELF) });
            let name = Some(Name::try_from(isa)?); // For now assume super
            let env = id_from_var(&var, &name, &None, false, ctx, constr, env)?;

            constrain_class_body(conditions, isa, &env, ctx, constr)
        }
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
    let name = StringName::try_from(ty)?;
    let class_env = env.in_class(&name);
    gen_vec(statements, &class_env, true, ctx, constr)?;

    // preserve mapping of self outside class to prevent contamination
    if let Some(self_map) = constr.var_mapping.get(SELF) {
        Ok(env.override_mapping(SELF, *self_map))
    } else {
        Ok(env.clone())
    }
}
