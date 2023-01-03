use std::convert::TryFrom;
use std::ops::Deref;

use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::expected::Expect::*;
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::generate::{Constrained, gen_vec, generate};
use crate::check::constrain::generate::env::Environment;
use crate::check::context::{field, LookupClass};
use crate::check::context::Context;
use crate::check::name::Name;
use crate::check::name::string_name::StringName;
use crate::check::result::TypeErr;
use crate::common::position::Position;
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
    let class_name = StringName::try_from(ty.deref())?;
    let class_lvl = constr.new_set_in_class(&class_name);
    let class_ty_exp = Type { name: Name::from(&class_name) };

    let mut env_with_class_fields = env.in_class(&Expected::new(ty.pos, &class_ty_exp));
    for field in ctx.class(&class_name, ty.pos)?.fields {
        env_with_class_fields = property_from_field(ty.pos, &field, &class_name, &env_with_class_fields, constr)?;
    }

    constr.add(
        "class body",
        &Expected::try_from((&AST { pos: ty.pos, node: Node::new_self() }, &env_with_class_fields.var_mappings))?,
        &Expected::new(ty.pos, &class_ty_exp),
    );

    gen_vec(statements, &env_with_class_fields, true, ctx, constr)?;
    constr.exit_set_to(class_lvl, ty.pos)?;
    Ok(env_with_class_fields.clone())
}

/// Generate constraint for a given field.
pub fn property_from_field(
    pos: Position,
    field: &field::Field,
    class: &StringName,
    env: &Environment,
    constr: &mut ConstrBuilder,
) -> Constrained {
    let node = Node::PropertyCall {
        instance: Box::new(AST { pos, node: Node::new_self() }),
        property: Box::new(AST { pos, node: Node::Id { lit: field.name.clone() } }),
    };
    let property_call = Expected::try_from((&AST::new(pos, node), &env.var_mappings))?;
    let field_ty = Expected::new(pos, &Type { name: field.ty.clone() });

    let env = env.insert_var(field.mutable, &field.name, &field_ty);
    constr.add("class field type", &field_ty, &property_call);

    let access = Expected::new(pos, &Access {
        entity: Box::new(Expected::new(pos, &Type { name: Name::from(class) })),
        name: Box::new(Expected::new(pos, &Field { name: field.name.clone() })),
    });

    constr.add("class field access", &property_call, &access);
    Ok(env)
}
