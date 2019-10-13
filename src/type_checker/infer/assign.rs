use std::convert::TryFrom;
use std::ops::Deref;

use crate::parser::ast::{Node, AST};
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::context::Context;
use crate::type_checker::environment::infer_type::InferType;
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::TypeErr;

pub fn infer_assign(ast: &AST, env: &Environment, ctx: &Context, state: &State) -> InferResult {
    match &ast.node {
        Node::Id { lit } => Ok((InferType::from(&env.lookup(lit, &ast.pos)?), env.clone())),
        Node::Reassign { left, right } => {
            let (left_ty, env) = infer(left, env, ctx, state)?;
            let (right_ty, env) = infer(right, &env, ctx, state)?;

            let left_expr = left_ty.expr_ty(&ast.pos)?;
            // TODO reevaluate how we deal with mutable (should this be expression level?)
            let right_expr = right_ty.expr_ty(&ast.pos)?;
            if left_expr == right_expr {
                Ok((InferType::new().add_raises(&left_ty).add_raises(&right_ty), env))
            } else {
                Err(vec![TypeErr::new(
                    &ast.pos,
                    &format!("Types must be equal, should be {}, was {}", left_expr, right_expr)
                )])
            }
        }
        // TODO use forward and private
        Node::VariableDef { id_maybe_type, expression, .. } => match &id_maybe_type.node {
            // TODO Check whether mutable
            // TODO use system for tuples of ids
            Node::IdType { _type, id, mutable } => {
                let id = match &id.node {
                    Node::Id { lit } => lit.clone(),
                    _ => return Err(vec![TypeErr::new(&id.pos, "Expected identifier")])
                };

                let (ty, env) = match (_type, expression) {
                    (Some(ty_name), Some(expr)) => {
                        let infer_type =
                            ctx.lookup(&TypeName::try_from(ty_name.deref())?, &ty_name.pos)?;
                        let (other_ty, env) = infer(expr, env, ctx, state)?;
                        if infer_type != other_ty {
                            return Err(vec![TypeErr::new(
                                &expr.pos,
                                "Expression type does not match annotated type"
                            )]);
                        }

                        (other_ty, env)
                    }
                    (None, Some(expr)) => infer(expr, env, ctx, state)?,
                    (Some(ty_name), None) => (
                        ctx.lookup(&TypeName::try_from(ty_name.deref())?, &ty_name.pos)?,
                        env.clone()
                    ),
                    (None, None) => return Err(vec![TypeErr::new(&ast.pos, "Cannot infer type")])
                };

                let env = env.insert(id.as_str(), *mutable, &ty.expr_ty(&ast.pos)?)?;
                Ok((InferType::new(), env))
            }
            _ => Err(vec![TypeErr::new(&ast.pos, "Expected identifier")])
        },
        Node::FunArg { .. } => unimplemented!(),
        Node::FunDef { .. } => unimplemented!(),

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected variable manipulation")])
    }
}
