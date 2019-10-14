use crate::parser::ast::{Node, AST};
use crate::type_checker::context::ty::concrete;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::context::Context;
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::TypeErr;

pub fn infer_boolean_op(ast: &AST, env: &Environment, ctx: &Context, state: &State) -> InferResult {
    match &ast.node {
        Node::Eq { left, right }
        | Node::Neq { left, right }
        | Node::Is { left, right }
        | Node::IsN { left, right }
        | Node::IsA { left, right }
        | Node::IsNA { left, right } => {
            let (left_ty, env) = infer(left, env, ctx, state)?;
            let (right_ty, env) = infer(right, &env, ctx, &state)?;
            left_ty.expr_ty(&left.pos)?;
            right_ty.expr_ty(&right.pos)?;
            Ok((
                ctx.lookup(&TypeName::from(concrete::BOOL_PRIMITIVE), &ast.pos)?
                    .add_raises(&left_ty)
                    .add_raises(&right_ty),
                env
            ))
        }
        Node::And { left, right } | Node::Or { left, right } => {
            let (left_ty, env) = infer(left, env, ctx, state)?;
            let (right_ty, env) = infer(right, &env, ctx, &state)?;

            if left_ty != ctx.lookup(&TypeName::from(concrete::BOOL_PRIMITIVE), &ast.pos)? {
                return Err(vec![TypeErr::new(
                    &left.pos,
                    &format!("Expected {}, was {}", concrete::BOOL_PRIMITIVE, left_ty)
                )]);
            }
            if right_ty != ctx.lookup(&TypeName::from(concrete::BOOL_PRIMITIVE), &ast.pos)? {
                return Err(vec![TypeErr::new(
                    &right.pos,
                    &format!("Expected {}, was {}", concrete::BOOL_PRIMITIVE, right_ty)
                )]);
            }

            Ok((
                ctx.lookup(&TypeName::new(concrete::BOOL_PRIMITIVE, &vec![]), &ast.pos)?
                    .add_raises(&left_ty)
                    .add_raises(&right_ty),
                env
            ))
        }
        Node::Not { expr } => {
            let (ty, env) = infer(expr, env, ctx, state)?;
            if ty != ctx.lookup(&TypeName::from(concrete::BOOL_PRIMITIVE), &ast.pos)? {
                return Err(vec![TypeErr::new(
                    &expr.pos,
                    &format!("Expected {}, was {}", concrete::BOOL_PRIMITIVE, ty)
                )]);
            }
            Ok((ty, env))
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected boolean operation")])
    }
}
