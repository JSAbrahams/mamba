use crate::parser::ast::{Node, AST};
use crate::type_checker::context::concrete::function::Function;
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

        Node::Add { left, right } => overrides(left, right, env, ctx, state, Function::ADD),
        Node::AddU { expr } => unimplemented!(),
        Node::Sub { left, right } => overrides(left, right, env, ctx, state, Function::SUB),
        Node::SubU { .. } => unimplemented!(),
        Node::Mul { left, right } => overrides(left, right, env, ctx, state, Function::MUL),
        Node::Div { left, right } => overrides(left, right, env, ctx, state, Function::DIV),
        Node::FDiv { left, right } => overrides(left, right, env, ctx, state, Function::FDIV),
        Node::Mod { left, right } => overrides(left, right, env, ctx, state, Function::MOD),
        Node::Pow { left, right } => overrides(left, right, env, ctx, state, Function::POW),
        Node::Sqrt { .. } => unimplemented!(),
        Node::Le { left, right } => overrides(left, right, env, ctx, state, Function::LE),
        Node::Ge { left, right } => overrides(left, right, env, ctx, state, Function::GE),
        Node::Leq { left, right } => overrides(left, right, env, ctx, state, Function::LEQ),
        Node::Geq { left, right } => overrides(left, right, env, ctx, state, Function::GEQ),
        Node::Eq { left, right } => overrides(left, right, env, ctx, state, Function::LE),
        Node::Neq { left, right } => overrides(left, right, env, ctx, state, Function::NEQ),

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected operation")])
    }
}

fn overrides(
    left: &AST,
    right: &AST,
    env: &Environment,
    ctx: &Context,
    state: &State,
    overrides: &str
) -> InferResult {
    let (left_expr_ty, left_env) = infer(left, env, ctx, state)?;
    let (right_expr_ty, right_env) = infer(right, &left_env, ctx, &state)?;

    if left_expr_ty == right_expr_ty {
        if let Some(type_name) = left_expr_ty.expr_type {
            if type_name.types.len() > 1 {
                Err(vec![TypeErr::new(&left.pos, "Tuple cannot override operator")])
            } else if let Some(ty) = type_name.types.get(0) {
                // TODO check if type overrides operator
                unimplemented!()
            } else {
                // This should, in theory, never be returned
                Err(vec![TypeErr::new(&left.pos, "Must be expression")])
            }
        } else {
            Err(vec![TypeErr::new(&left.pos, "Must be expression")])
        }
    } else {
        Err(vec![TypeErr::new(&left.pos.union(&right.pos), "Types must be equal")])
    }
}
