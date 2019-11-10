use std::collections::HashSet;

use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::context::ty::concrete;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::context::Context;
use crate::type_checker::environment::expression_type::actual_type::ActualType;
use crate::type_checker::environment::expression_type::nullable_type::NullableType;
use crate::type_checker::environment::expression_type::ExpressionType;
use crate::type_checker::environment::infer_type::InferType;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::{TypeErr, TypeResult};

pub fn infer_coll(ast: &AST, env: &Environment, ctx: &Context) -> InferResult {
    match &ast.node {
        Node::Tuple { elements } => {
            let mut env = env.clone();
            let mut types = vec![];
            let mut raises = HashSet::new();
            for element in elements {
                let (ty, new_env) = infer(element, &env, ctx)?;
                types.push(ty.expr_ty(&element.pos)?);
                raises = raises.union(&ty.raises).cloned().collect();
                env = new_env;
            }

            let actual_ty = ActualType::Tuple { types };
            let nullable_ty = NullableType::new(false, &actual_ty);
            let expr_ty = ExpressionType::from(&nullable_ty);
            let ty = InferType::from(&expr_ty);
            Ok((ty.union_raises(&raises), env))
        }
        Node::Set { elements } => collection(&concrete::SET, ast, elements, env, ctx),
        Node::List { elements } => collection(&concrete::LIST, ast, elements, env, ctx),

        Node::ListBuilder { .. } => Err(vec![TypeErr::new(&ast.pos, "Not implemented")]),
        Node::SetBuilder { .. } => Err(vec![TypeErr::new(&ast.pos, "Not implemented")]),

        Node::In { left, right } => {
            let (ty, env) = infer(left, env, ctx)?;
            let (col_ty, env) = infer(right, &env, ctx)?;

            // TODO check that right is set or list
            // TODO check list or set type is left type

            let bool_ty = ctx.lookup(&TypeName::from(concrete::BOOL_PRIMITIVE), &ast.pos)?;
            Ok((InferType::from(&bool_ty).add_raises(&ty).add_raises(&col_ty), env))
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected collection")])
    }
}

fn collection(
    col: &str,
    ast: &AST,
    elements: &Vec<AST>,
    env: &Environment,
    ctx: &Context
) -> InferResult {
    let mut env = env.clone();
    let mut types = vec![];
    let mut raises = HashSet::new();
    for element in elements {
        let (ty, new_env) = infer(element, &env, ctx)?;
        types.push(ty.expr_ty(&element.pos)?);
        raises = raises.union(&ty.raises).cloned().collect();
        env = new_env;
    }

    let first = types.first().cloned();
    for ty in types {
        // TODO use union instead of throwing an error
        if first.clone().unwrap_or_else(|| unreachable!()) != ty {
            return Err(vec![TypeErr::new(&ast.pos, "Set types must all be same")]);
        }
    }

    let generics = vec![match first {
        Some(first) => TypeName::from(&first),
        None => TypeName::from(&ctx.lookup(&TypeName::from(concrete::ANY), &ast.pos)?)
    }];

    let ty = ctx.lookup(&TypeName::new(col, &generics), &ast.pos)?;
    let coll_ty = InferType::from(&ty);
    Ok((coll_ty.union_raises(&raises), env))
}

pub fn iterable_generic(
    expr_ty: &ExpressionType,
    ctx: &Context,
    env: &Environment,
    pos: &Position
) -> TypeResult<ExpressionType> {
    match expr_ty {
        ExpressionType::Single { ty } => match &ty.actual_ty() {
            ActualType::Single { ty } => {
                let iterable = ty
                    .fun("__iter__", &vec![], pos)?
                    .ty()
                    .ok_or(TypeErr::new(pos, &format!("Cannot iterate over {}", expr_ty)))?;
                let next_ty = ctx
                    .lookup(&iterable, pos)?
                    .fun("__next__", &vec![], pos)?
                    .ty()
                    .ok_or(TypeErr::new(pos, &format!("Cannot iterate over {}", expr_ty)))?;
                ctx.lookup(&next_ty, pos)
            }
            ActualType::Tuple { types } => {
                let first_ty = types.first();
                let mut first =
                    first_ty.ok_or(TypeErr::new(pos, &format!("Cannot infer type")))?.clone();
                for ty in types {
                    first = first.union(ty);
                }
                Ok(first)
            }
            ActualType::AnonFun { .. } => Err(vec![TypeErr::new(pos, "Must be single or tuple")])
        },
        ExpressionType::Union { union } => {
            let union: Vec<ExpressionType> = union
                .iter()
                .map(|null_ty| iterable_generic(&ExpressionType::from(null_ty), ctx, env, pos))
                .collect::<Result<_, _>>()?;
            let mut first = union
                .first()
                .cloned()
                .ok_or(vec![TypeErr::new(pos, "Cannot infer type of iterable")])?;
            for ty in union {
                first = first.union(&ty);
            }
            Ok(first)
        }
    }
}
