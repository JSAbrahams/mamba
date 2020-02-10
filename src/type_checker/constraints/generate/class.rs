use std::convert::TryFrom;
use std::ops::Deref;

use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::checker_result::TypeErr;
use crate::type_checker::constraints::constraint::builder::ConstrBuilder;
use crate::type_checker::constraints::constraint::expected::Expect::*;
use crate::type_checker::constraints::constraint::expected::Expected;
use crate::type_checker::constraints::generate::{gen_vec, generate};
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::field::concrete::Field;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::ty_name::TypeName;

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

    let class_name = TypeName::try_from(ty.deref())?;
    res.0.new_set_in_class(true, &class_name);
    let class_ty_exp = Type { type_name: class_name.clone() };
    res.1 = res.1.in_class(&Expected::new(&ty.pos, &class_ty_exp));

    let all_fields = ctx.lookup(&class_name, &ty.pos)?.fields(&ty.pos)?;
    for fields in all_fields {
        for field in fields {
            res = property_from_field(&ty.pos, &field, &class_name, &res.1, &mut res.0)?;
        }
    }

    res.0.add(
        &Expected::from(&AST { pos: ty.pos.clone(), node: Node::_Self }),
        &Expected::new(&ty.pos, &class_ty_exp)
    );

    res = gen_vec(statements, &res.1, ctx, &mut res.0)?;
    res.0.exit_set(&ty.pos)?;
    Ok((res.0, env.clone()))
}

/// Generate constraint for a given field.
pub fn property_from_field(
    pos: &Position,
    field: &Field,
    class_ty: &TypeName,
    env: &Environment,
    constr: &mut ConstrBuilder
) -> Constrained {
    // TODO generate constraints are part of interface
    // TODO add constraint for mutable field
    let field_ty = field.ty.clone().ok_or_else(|| {
        let msg = format!(
            "{} did not have a type annotation.\nCurrently, all fields must have a type.\nIn \
             future, we will infer these types.",
            field
        );
        TypeErr::new(&pos, &msg)
    })?;

    let node = Node::PropertyCall {
        instance: Box::new(AST { pos: pos.clone(), node: Node::_Self }),
        property: Box::new(AST { pos: pos.clone(), node: Node::Id { lit: field.name.clone() } })
    };
    let property_call = Expected::from(&AST::new(&pos, node));
    let field_ty = Expected::new(&pos, &Type { type_name: field_ty });

    let env = env.insert_var(field.mutable, &field.name, &field_ty);
    constr.add(&field_ty, &property_call);

    constr.add(
        &Expected::new(&pos, &Type { type_name: class_ty.clone() }),
        &Expected::new(&pos, &HasField { name: field.name.clone() })
    );

    Ok((constr.clone(), env))
}
