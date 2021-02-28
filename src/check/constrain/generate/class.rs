use std::convert::TryFrom;
use std::ops::Deref;

use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::expected::Expect::*;
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::generate::{gen_vec, generate};
use crate::check::constrain::Constrained;
use crate::check::context::name::{DirectName, NameUnion};
use crate::check::context::Context;
use crate::check::context::{field, LookupClass};
use crate::check::env::Environment;
use crate::check::result::TypeErr;
use crate::common::position::Position;
use crate::parse::ast::{Node, AST};

pub fn gen_class(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder
) -> Constrained {
    match &ast.node {
        Node::Class { body: Some(body), ty, .. } | Node::TypeDef { body: Some(body), ty, .. } =>
            match &body.node {
                Node::Block { statements } =>
                    constrain_class_body(statements, ty, env, ctx, constr),
                _ => Err(vec![TypeErr::new(&body.pos, "Expected code block")])
            },
        Node::Class { .. } | Node::TypeDef { .. } => Ok((constr.clone(), env.clone())),

        Node::TypeAlias { conditions, isa, .. } =>
            constrain_class_body(conditions, isa, env, ctx, constr),
        Node::Condition { cond, el: Some(el) } => {
            let (mut constr, env) = generate(cond, env, ctx, constr)?;
            generate(el, &env, ctx, &mut constr)
        }
        Node::Condition { cond, .. } => generate(cond, env, ctx, constr),

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected class or type definition")])
    }
}

pub fn constrain_class_body(
    statements: &[AST],
    ty: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder
) -> Constrained {
    let mut res = (constr.clone(), env.clone());

    let class_name = DirectName::try_from(ty.deref())?;
    res.0.new_set_in_class(true, &class_name);
    let class_ty_exp = Type { name: NameUnion::from(&class_name) };
    res.1 = res.1.in_class(&Expected::new(&ty.pos, &class_ty_exp));

    for field in ctx.class(&class_name, &ty.pos)?.fields {
        res = property_from_field(&ty.pos, &field, &class_name, &mut res.1, &mut res.0)?;
    }

    res.0.add(
        "class body",
        &Expected::try_from((&AST { pos: ty.pos.clone(), node: Node::_Self }, env))?,
        &Expected::new(&ty.pos, &class_ty_exp)
    );

    res = gen_vec(statements, &res.1, ctx, &res.0)?;
    res.0.exit_set(&ty.pos)?;
    Ok((res.0, env.clone()))
}

/// Generate constraint for a given field.
pub fn property_from_field(
    pos: &Position,
    field: &field::Field,
    class: &DirectName,
    env: &mut Environment,
    constr: &mut ConstrBuilder
) -> Constrained {
    // TODO generate constraints are part of interface
    // TODO add constraint for mutable field
    let node = Node::PropertyCall {
        instance: Box::new(AST { pos: pos.clone(), node: Node::_Self }),
        property: Box::new(AST { pos: pos.clone(), node: Node::Id { lit: field.name.clone() } })
    };
    let property_call = Expected::try_from((&AST::new(&pos, node), &env.clone()))?;
    let field_ty = Expected::new(&pos, &Type { name: field.ty.clone() });

    let env = env.insert_var(field.mutable, &field.name, &field_ty);
    constr.add("field property", &field_ty, &property_call);

    let access = Expected::new(&pos, &Access {
        entity: Box::new(Expected::new(&pos, &Type { name: NameUnion::from(class) })),
        name:   Box::new(Expected::new(&pos, &Field { name: field.name.clone() }))
    });

    constr.add("field property", &property_call, &access);
    Ok((constr.clone(), env))
}
