use std::convert::TryFrom;
use std::ops::Deref;

use crate::parser::ast::{Node, AST};
use crate::type_checker::checker_result::TypeErr;
use crate::type_checker::constraints::constraint::builder::ConstrBuilder;
use crate::type_checker::constraints::constraint::expected::Expect::*;
use crate::type_checker::constraints::constraint::expected::Expected;
use crate::type_checker::constraints::generate::{gen_vec, generate};
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::Context;
use crate::type_checker::environment::name::Identifier;
use crate::type_checker::environment::Environment;
use crate::type_checker::ty_name::TypeName;

pub fn gen_class(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder
) -> Constrained {
    match &ast.node {
        Node::Class { body: Some(body), args, ty, .. } => match &body.node {
            Node::Block { statements } => {
                constr.new_set(true);
                let mut res = (constr.clone(), env.clone());
                let type_name = TypeName::try_from(ty.deref())?;
                let all_fields = ctx.lookup(&type_name, &ty.pos)?.fields(&ty.pos)?;

                for fields in all_fields {
                    for field in fields {
                        let var = AST { pos: ty.pos.clone(), node: Node::Id { lit: field.name } };

                        // TODO generate constraints are part of interface
                        let field_ty = field.ty.ok_or_else(|| {
                            TypeErr::new(
                                &ty.pos,
                                "Currently, all fields must have a type.\n In future, we will \
                                 have infer these types."
                            )
                        })?;
                        let field_ty_exp = Expected::new(&ty.pos, &Type { type_name: field_ty });
                        res = property_from_field(
                            field.mutable,
                            &var,
                            &type_name,
                            &field_ty_exp,
                            &res.1,
                            &mut res.0
                        )?;

                        if field.mutable {
                            let var_exp = Expected::from(&var);
                            res.0.add(&var_exp, &Expected::new(&ty.pos, &Mutable))
                        }
                    }
                }

                res.1 = res.1.in_class_new(&Type { type_name });
                res = gen_vec(statements, &res.1, ctx, &mut res.0)?;

                res.0.exit_set(&ast.pos)?;
                Ok(res)
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
            let (mut constr, env) = generate(cond, env, ctx, constr)?;
            generate(el, &env, ctx, &mut constr)
        }
        Node::Condition { cond, .. } => generate(cond, env, ctx, constr),

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected class or type definition")])
    }
}

/// Generate constraint for a given field.
///
/// arg_exp can be either expected type or the default value.
/// Generates constraint such that a property access of self and the given field
/// is constrained on the type. Also generates a constraint such that the type
/// of self (given as class_ty) HasField with given name. Uses Identifier to
/// generate fields from given AST.
pub fn property_from_field(
    mutable: bool,
    field: &AST,
    class_ty: &TypeName,
    arg_exp: &Expected,
    env: &Environment,
    constr: &mut ConstrBuilder
) -> Constrained {
    let identifier = Identifier::try_from(field)?;
    let mut res = (constr.clone(), env.clone());
    for (_, f_name) in &identifier.fields() {
        let node = Node::PropertyCall {
            instance: Box::new(AST { pos: field.pos.clone(), node: Node::_Self }),
            property: Box::new(AST {
                pos:  field.pos.clone(),
                node: Node::Id { lit: f_name.clone() }
            })
        };

        let property_exp = Expected::from(&AST::new(&field.pos, node));
        res.1 = res.1.insert_new(mutable, f_name, &property_exp.expect);
        res.0.add(&arg_exp, &property_exp);
        res.0.add(
            &Expected::new(&field.pos, &Type { type_name: class_ty.clone() }),
            &Expected::new(&field.pos, &HasField { name: f_name.clone() })
        )
    }

    Ok(res)
}
