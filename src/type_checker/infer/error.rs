use std::collections::HashSet;
use std::convert::TryFrom;
use std::iter::FromIterator;
use std::ops::Deref;

use crate::parser::ast::{Node, AST};
use crate::type_checker::context::ty::concrete;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::infer_type::InferType;
use crate::type_checker::type_name::actual::ActualTypeName;
use crate::type_checker::type_name::TypeName;
use crate::type_checker::type_result::TypeErr;
use crate::type_checker::util::comma_delimited;

pub fn infer_error(ast: &AST, env: &Environment, ctx: &Context) -> InferResult {
    match &ast.node {
        Node::Raise { error } => {
            if !env.state.in_function {
                return Err(vec![TypeErr::new(&ast.pos, "Raise cannot be outside function")]);
            }

            let (ty, env) = infer(error, env, ctx)?;
            let actual_ty = ty.expr_ty(&error.pos)?.single(&error.pos)?.actual_ty();
            let set = HashSet::from_iter(vec![ActualTypeName::from(&actual_ty)].into_iter());

            let exception = ctx.lookup(&TypeName::from(concrete::EXCEPTION), &ast.pos)?;
            Ok((InferType::from(&exception).union_raises(&set).add_raises(&ty), env))
        }
        Node::Raises { expr_or_stmt, errors } => {
            let (ty, env) = infer(expr_or_stmt, env, ctx)?;
            let errors = errors.iter().map(ActualTypeName::try_from).collect::<Result<_, _>>()?;
            if ty.raises.is_superset(&errors) {
                Ok((ty, env))
            } else {
                let new_set: HashSet<_> = errors.difference(&ty.raises).collect();
                let msg = format!(
                    "The following errors are mentioned but never raised: {}",
                    comma_delimited(new_set)
                );
                Err(vec![TypeErr::new(&expr_or_stmt.pos, &msg)])
            }
        }

        Node::With { resource, alias, expr } => {
            let (resource_ty, mut inner_env) = infer(resource, env, ctx)?;

            if let Some((alias, mutable, ty)) = alias {
                let type_name =
                    if let Some(ty) = ty { Some(TypeName::try_from(ty.deref())?) } else { None };
                let alias = if let Node::Id { lit } = &alias.node {
                    lit.clone()
                } else {
                    return Err(vec![TypeErr::new(&alias.pos, "Expected identifier")]);
                };

                let expr_ty = resource_ty.expr_ty(&resource.pos)?;
                if let Some(type_name) = type_name {
                    if type_name != TypeName::from(&expr_ty) {
                        let msg =
                            format!("Expected {} but was {}", type_name, TypeName::from(&expr_ty));
                        return Err(vec![TypeErr::new(&resource.pos, &msg)]);
                    }
                }

                if let Node::Id { lit } = &resource.node {
                    inner_env.remove(&lit);
                }
                inner_env.insert(&alias, *mutable, &expr_ty);
            }

            let (infer_ty, _) = infer(expr, &inner_env, ctx)?;
            Ok((InferType::default().union_raises(&infer_ty.raises), env.clone()))
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected error")])
    }
}
