use crate::parser::ast::{Node, AST};
use crate::type_checker::context::generic::type_name::GenericTypeName;
use crate::type_checker::context::{concrete, Context};
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::infer_type::InferType;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::TypeErr;

pub fn infer_boolean_op(ast: &AST, env: &Environment, ctx: &Context, state: &State) -> InferResult {
    match &ast.node {
        Node::Eq { left, right }
        | Node::Neq { left, right }
        | Node::Is { left, right }
        | Node::IsN { left, right }
        | Node::Neq { left, right }
        | Node::IsA { left, right }
        | Node::IsNA { left, right } => {
            let (left_ty, left_env) = infer(left, env, ctx, state)?;
            let (right_ty, right_env) = infer(right, &left_env, ctx, &state)?;

            let left_expr_ty = left_ty.expr_ty(&left.pos)?;
            let right_expr_ty = right_ty.expr_ty(&right.pos)?;

            Ok((
                InferType::from(ctx.lookup(&GenericTypeName::new(concrete::BOOL), &ast.pos)?)
                    .raises(left_ty.raises)
                    .raises(right_ty.raises),
                env.clone()
            ))
        }

        Node::And { left, right } | Node::Or { left, right } => {
            let (left_ty, left_env) = infer(left, env, ctx, state)?;
            let (right_ty, right_env) = infer(right, &left_env, ctx, &state)?;

            let left_expr_ty = left_ty.expr_ty(&ast.pos)?;
            let actual_ty = left_expr_ty.get_actual_ty(&ast.pos)?;
            let ty = actual_ty.ty(&ast.pos)?;
            if ty.name != concrete::BOOL_PRIMITIVE {
                return Err(vec![TypeErr::new(&left.pos, "Expected boolean")]);
            }

            let right_expr_ty = right_ty.expr_ty(&ast.pos)?;
            let actual_ty = right_expr_ty.get_actual_ty(&ast.pos)?;
            let ty = actual_ty.ty(&ast.pos)?;
            if ty.name != concrete::BOOL_PRIMITIVE {
                return Err(vec![TypeErr::new(&right.pos, "Expected boolean")]);
            }

            Ok((
                InferType::from(ctx.lookup(&GenericTypeName::new(concrete::BOOL), &ast.pos)?)
                    .raises(left_ty.raises)
                    .raises(right_ty.raises),
                env.clone()
            ))
        }

        Node::Not { expr } => {
            let (infer_ty, env) = infer(expr, env, ctx, state)?;
            let expr_ty = infer_ty.expr_ty(&ast.pos)?;
            let actual_ty = expr_ty.get_actual_ty(&ast.pos)?;
            let ty = actual_ty.ty(&ast.pos)?;
            if ty.name != concrete::BOOL_PRIMITIVE {
                return Err(vec![TypeErr::new(&expr.pos, "Expected boolean")]);
            }

            Ok((infer_ty, env))
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected boolean operation")])
    }
}
