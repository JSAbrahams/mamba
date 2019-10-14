use std::collections::HashSet;

use crate::common::position::Position;
use crate::parser::ast::{AST, Node};
use crate::type_checker::context::Context;
use crate::type_checker::context::ty::concrete;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::environment::Environment;
use crate::type_checker::environment::expression_type::actual_type::ActualType;
use crate::type_checker::environment::expression_type::ExpressionType;
use crate::type_checker::environment::expression_type::mutable_type::NullableType;
use crate::type_checker::environment::infer_type::InferType;
use crate::type_checker::environment::state::State;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::{TypeErr, TypeResult};

pub fn infer_coll(ast: &AST, env: &Environment, ctx: &Context, state: &State) -> InferResult {
    match &ast.node {
        Node::Tuple { elements } => {
            let mut env = env.clone();
            let mut types = vec![];
            let mut raises = HashSet::new();
            for element in elements {
                let (ty, new_env) = infer(element, &env, ctx, state)?;
                types.push(ty.expr_ty(&element.pos)?);
                raises = raises.union(&ty.raises).cloned().collect();
                env = new_env;
            }

            let actual_ty = ActualType::Tuple { types };
            let mutable_ty = NullableType::from(&actual_ty);
            let expr_ty = ExpressionType::from(&mutable_ty);
            let ty = InferType::from(&expr_ty);
            Ok((ty.union_raises(&raises), env))
        }
        Node::Set { elements } => collection(&concrete::SET, ast, elements, env, ctx, state),
        Node::List { elements } => collection(&concrete::LIST, ast, elements, env, ctx, state),

        Node::ListBuilder { .. } => Err(vec![TypeErr::new(&ast.pos, "Not implemented")]),
        Node::SetBuilder { .. } => Err(vec![TypeErr::new(&ast.pos, "Not implemented")]),

        Node::In { left, right } => {
            let (ty, env) = infer(left, env, ctx, state)?;
            let (col_ty, env) = infer(right, &env, ctx, state)?;

            // TODO check that right is set or list
            // TODO check list or set type is left type

            Ok((
                ctx.lookup(&TypeName::new(concrete::BOOL_PRIMITIVE, &vec![]), &ast.pos)?
                    .add_raises(&ty)
                    .add_raises(&col_ty),
                env
            ))
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected collection")])
    }
}

fn collection(
    col: &str,
    ast: &AST,
    elements: &Vec<AST>,
    env: &Environment,
    ctx: &Context,
    state: &State,
) -> InferResult {
    let mut env = env.clone();
    let mut types = vec![];
    let mut raises = HashSet::new();
    for element in elements {
        let (ty, new_env) = infer(element, &env, ctx, state)?;
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
        Some(first) => first.clone(),
        None => ctx.lookup(&TypeName::new(concrete::ANY, &vec![]), &ast.pos)?.expr_ty(&ast.pos)?
    }];
    let ty = ctx.lookup(&TypeName::from((col, &generics)), &ast.pos)?;

    Ok((ty.union_raises(&raises), env))
}

pub fn iterable_generic(
    expr_ty: &ExpressionType,
    ctx: &Context,
    pos: &Position,
) -> TypeResult<ExpressionType> {
    match expr_ty {
        ExpressionType::Single { mut_ty } => match &mut_ty.actual_ty {
            ActualType::Single { ty } => {
                let iterable = ty.fun("__iter__", &vec![], false, pos)?.ty().ok_or(
                    TypeErr::new(pos, &format!("Cannot iterate over {}", expr_ty))
                )?;
                let next_ty = ctx.lookup(&iterable, pos)?.expr_ty(pos)?.fun("__next__", &vec![], false, pos)?.ty().ok_or(
                    TypeErr::new(pos, &format!("Cannot iterate over {}", expr_ty))
                )?;
                ctx.lookup(&next_ty, pos)?.expr_ty(pos).map_err(|e| vec![e])
            }
            ActualType::Tuple { .. } => unimplemented!(),
            ActualType::AnonFun { .. } => Err(vec![TypeErr::new(pos, "Must be single or tuple")])
        },
        ExpressionType::Union { .. } => unimplemented!()
    }
}
