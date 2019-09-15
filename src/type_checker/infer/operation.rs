use crate::parser::ast::{Node, AST};
use crate::type_checker::context::Context;
use crate::type_checker::environment::expression_type::ExpressionType;
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::TypeErr;

pub fn infer_operation(
    ast: &Box<AST>,
    env: &Environment,
    ctx: &Context,
    state: &State
) -> InferResult {
    match &ast.node {
        Node::_Self => Ok((None, env.clone(), state.clone())),
        Node::AddOp => Ok((None, env.clone(), state.clone())),
        Node::SubOp => Ok((None, env.clone(), state.clone())),
        Node::SqrtOp => Ok((None, env.clone(), state.clone())),
        Node::MulOp => Ok((None, env.clone(), state.clone())),
        Node::FDivOp => Ok((None, env.clone(), state.clone())),
        Node::DivOp => Ok((None, env.clone(), state.clone())),
        Node::PowOp => Ok((None, env.clone(), state.clone())),
        Node::ModOp => Ok((None, env.clone(), state.clone())),
        Node::EqOp => Ok((None, env.clone(), state.clone())),
        Node::LeOp => Ok((None, env.clone(), state.clone())),
        Node::GeOp => Ok((None, env.clone(), state.clone())),

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

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected operation")])
    }
}

fn overrides(
    left: &Box<AST>,
    right: &Box<AST>,
    env: &Environment,
    ctx: &Context,
    state: &State,
    overrides: &str
) -> InferResult {
    let (left_expr_ty, left_env) = infer(left, env, ctx, state)?;
    let (right_expr_ty, right_env) = infer(right, &left_env, ctx, state)?;

    if left_expr_ty == right_expr_ty {
        if let Some(type_name) = left_expr_ty.ty {
            let ty = ctx.lookup(&type_name, &left.pos)?;
            if ty.overrides_function(Function::Le) {
                Ok((Some(ExpressionType::new(&Some(type_name))), env.clone(), state.clone()))
            } else {
                Err(vec![TypeErr::new(&left.pos, "Type does not define operator")])
            }
        } else {
            Err(vec![TypeErr::new(&left.pos, "Must have type")])
        }
    } else {
        Err(vec![TypeErr::new(&ast.pos, "Types must be equal")])
    }
}
