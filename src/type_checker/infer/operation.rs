use crate::parser::ast::{Node, AST};
use crate::type_checker::context::concrete::function;
use crate::type_checker::context::Context;
use crate::type_checker::environment::infer_type::InferType;
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::TypeErr;

pub fn infer_op(ast: &AST, env: &Environment, ctx: &Context, state: &State) -> InferResult {
    match &ast.node {
        Node::_Self => Ok((InferType::new(), env.clone())),
        Node::AddOp => Ok((InferType::new(), env.clone())),
        Node::SubOp => Ok((InferType::new(), env.clone())),
        Node::SqrtOp => Ok((InferType::new(), env.clone())),
        Node::MulOp => Ok((InferType::new(), env.clone())),
        Node::FDivOp => Ok((InferType::new(), env.clone())),
        Node::DivOp => Ok((InferType::new(), env.clone())),
        Node::PowOp => Ok((InferType::new(), env.clone())),
        Node::ModOp => Ok((InferType::new(), env.clone())),
        Node::EqOp => Ok((InferType::new(), env.clone())),
        Node::LeOp => Ok((InferType::new(), env.clone())),
        Node::GeOp => Ok((InferType::new(), env.clone())),

        Node::AddU { .. } => unimplemented!(),
        Node::SubU { .. } => unimplemented!(),
        Node::Sqrt { .. } => unimplemented!(),

        Node::Add { left, right } => override_op(left, right, function::ADD, env, ctx, state),
        Node::Sub { left, right } => override_op(left, right, function::SUB, env, ctx, state),
        Node::Mul { left, right } => override_op(left, right, function::MUL, env, ctx, state),
        Node::Div { left, right } => override_op(left, right, function::DIV, env, ctx, state),
        Node::FDiv { left, right } => override_op(left, right, function::FDIV, env, ctx, state),
        Node::Mod { left, right } => override_op(left, right, function::MOD, env, ctx, state),
        Node::Pow { left, right } => override_op(left, right, function::POW, env, ctx, state),
        Node::Le { left, right } => override_op(left, right, function::LE, env, ctx, state),
        Node::Ge { left, right } => override_op(left, right, function::GE, env, ctx, state),
        Node::Leq { left, right } => override_op(left, right, function::LEQ, env, ctx, state),
        Node::Geq { left, right } => override_op(left, right, function::GEQ, env, ctx, state),

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected operation")])
    }
}

fn override_op(
    left: &Box<AST>,
    right: &Box<AST>,
    overrides: &str,
    env: &Environment,
    ctx: &Context,
    state: &State
) -> InferResult {
    let (left_ty, left_env) = infer(left, env, ctx, state)?;
    let (right_ty, right_env) = infer(right, &left_env, ctx, &state)?;
    if left_ty.expr_ty(&left.pos) == right_ty.expr_ty(&right.pos) {
        let fun = left_ty.expr_ty(&left.pos).fun(
            overrides,
            &vec![right_ty.expr_ty(&right.pos)],
            &left.pos
        )?;
        Ok((left_ty.union(&right_ty, &left.pos)?, right_env))
    } else {
        Err(vec![TypeErr::new(&left.pos, "Types differ")])
    }
}
