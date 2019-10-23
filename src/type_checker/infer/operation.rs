use crate::parser::ast::{Node, AST};
use crate::type_checker::context::function::concrete;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::context::{function, Context};
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

        Node::AddU { expr } => unary_op(expr, function::concrete::ADD, env, ctx, state),
        Node::SubU { expr } => unary_op(expr, function::concrete::SUB, env, ctx, state),
        Node::Sqrt { expr } => unary_op(expr, function::concrete::SQRT, env, ctx, state),

        Node::Add { left, right } => op(left, right, concrete::ADD, env, ctx, state),
        Node::Sub { left, right } => op(left, right, concrete::SUB, env, ctx, state),
        Node::Mul { left, right } => op(left, right, concrete::MUL, env, ctx, state),
        Node::Div { left, right } => op(left, right, concrete::DIV, env, ctx, state),
        Node::FDiv { left, right } => op(left, right, concrete::FDIV, env, ctx, state),
        Node::Mod { left, right } => op(left, right, concrete::MOD, env, ctx, state),
        Node::Pow { left, right } => op(left, right, concrete::POW, env, ctx, state),
        Node::Le { left, right } => op(left, right, concrete::LE, env, ctx, state),
        Node::Ge { left, right } => op(left, right, concrete::GE, env, ctx, state),
        Node::Leq { left, right } => op(left, right, concrete::LEQ, env, ctx, state),
        Node::Geq { left, right } => op(left, right, concrete::GEQ, env, ctx, state),

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected operation")])
    }
}

fn unary_op(
    expr: &Box<AST>,
    overrides: &str,
    env: &Environment,
    ctx: &Context,
    state: &State
) -> InferResult {
    let (infer_type, env) = infer(expr, env, ctx, state)?;
    let fun = infer_type.expr_ty(&expr.pos)?.fun(overrides, &vec![], state.nullable, &expr.pos)?;
    match &fun.ty() {
        Some(fun_ty) => {
            let fun_ret_ty = ctx.lookup(fun_ty, &expr.pos)?;
            Ok((InferType::from(&fun_ret_ty).add_raises(&infer_type), env))
        }
        None => Ok((InferType::new().add_raises(&infer_type), env))
    }
}

fn op(
    left: &Box<AST>,
    right: &Box<AST>,
    overrides: &str,
    env: &Environment,
    ctx: &Context,
    state: &State
) -> InferResult {
    let (left_ty, left_env) = infer(left, env, ctx, state)?;
    let (right_ty, right_env) = infer(right, &left_env, ctx, &state)?;
    let fun = left_ty.expr_ty(&left.pos)?.fun(
        overrides,
        &vec![TypeName::from(&right_ty.expr_ty(&right.pos)?)],
        state.nullable,
        &left.pos
    )?;

    match &fun.ty() {
        Some(fun_ty) => {
            let expr_ty = ctx.lookup(fun_ty, &left.pos.union(&right.pos))?;
            Ok((InferType::from(&expr_ty).add_raises(&left_ty).add_raises(&right_ty), right_env))
        }
        None => Ok((InferType::new().add_raises(&left_ty).add_raises(&right_ty), right_env))
    }
}
