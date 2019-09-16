use crate::parser::ast::{Node, AST};
use crate::type_checker::context::Context;
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::infer_type::InferType;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::TypeErr;

pub fn infer_op(ast: &AST, env: &Environment, ctx: &Context, state: &State) -> InferResult {
    match &ast.node {
        Node::_Self => Ok((InferType::new(None), env.clone())),
        Node::AddOp => Ok((InferType::new(None), env.clone())),
        Node::SubOp => Ok((InferType::new(None), env.clone())),
        Node::SqrtOp => Ok((InferType::new(None), env.clone())),
        Node::MulOp => Ok((InferType::new(None), env.clone())),
        Node::FDivOp => Ok((InferType::new(None), env.clone())),
        Node::DivOp => Ok((InferType::new(None), env.clone())),
        Node::PowOp => Ok((InferType::new(None), env.clone())),
        Node::ModOp => Ok((InferType::new(None), env.clone())),
        Node::EqOp => Ok((InferType::new(None), env.clone())),
        Node::LeOp => Ok((InferType::new(None), env.clone())),
        Node::GeOp => Ok((InferType::new(None), env.clone())),

        Node::AddU { expr } => unimplemented!(),
        Node::SubU { .. } => unimplemented!(),
        Node::Sqrt { .. } => unimplemented!(),
        Node::Add { left, right }
        | Node::Sub { left, right }
        | Node::Mul { left, right }
        | Node::Div { left, right }
        | Node::FDiv { left, right }
        | Node::Mod { left, right }
        | Node::Pow { left, right }
        | Node::Le { left, right }
        | Node::Ge { left, right }
        | Node::Leq { left, right }
        | Node::Geq { left, right }
        | Node::Eq { left, right }
        | Node::Neq { left, right } => {
            let (left_expr_ty, left_env) = infer(left, env, ctx, state)?;
            let (right_expr_ty, right_env) = infer(right, &left_env, ctx, &state)?;

            if left_expr_ty == right_expr_ty {
                if let Some(type_name) = left_expr_ty.expr_type {
                    unimplemented!()
                } else {
                    Err(vec![TypeErr::new(&left.pos, "Must be expression")])
                }
            } else {
                Err(vec![TypeErr::new(&left.pos.union(&right.pos), "Types must be equal")])
            }
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected operation")])
    }
}
