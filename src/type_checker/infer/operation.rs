use crate::parser::ast::{Node, AST};
use crate::type_checker::context::function::concrete;
use crate::type_checker::context::{function, Context};
use crate::type_checker::environment::name::{match_name, Identifier};
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::infer_type::actual::ActualType;
use crate::type_checker::infer_type::expression::ExpressionType;
use crate::type_checker::infer_type::nullable::NullableType;
use crate::type_checker::infer_type::InferType;
use crate::type_checker::type_name::TypeName;
use crate::type_checker::type_result::TypeErr;
use std::convert::TryFrom;
use std::ops::Deref;

pub fn infer_op(ast: &AST, env: &Environment, ctx: &Context) -> InferResult {
    match &ast.node {
        Node::_Self => Ok((InferType::default(), env.clone())),
        Node::AddOp => Ok((InferType::default(), env.clone())),
        Node::SubOp => Ok((InferType::default(), env.clone())),
        Node::SqrtOp => Ok((InferType::default(), env.clone())),
        Node::MulOp => Ok((InferType::default(), env.clone())),
        Node::FDivOp => Ok((InferType::default(), env.clone())),
        Node::DivOp => Ok((InferType::default(), env.clone())),
        Node::PowOp => Ok((InferType::default(), env.clone())),
        Node::ModOp => Ok((InferType::default(), env.clone())),
        Node::EqOp => Ok((InferType::default(), env.clone())),
        Node::LeOp => Ok((InferType::default(), env.clone())),
        Node::GeOp => Ok((InferType::default(), env.clone())),

        Node::AddU { expr } => unary_op(expr, function::concrete::ADD, env, ctx),
        Node::SubU { expr } => unary_op(expr, function::concrete::SUB, env, ctx),
        Node::Sqrt { expr } => unary_op(expr, function::concrete::SQRT, env, ctx),

        Node::Add { left, right } => op(left, right, concrete::ADD, env, ctx),
        Node::Sub { left, right } => op(left, right, concrete::SUB, env, ctx),
        Node::Mul { left, right } => op(left, right, concrete::MUL, env, ctx),
        Node::Div { left, right } => op(left, right, concrete::DIV, env, ctx),
        Node::FDiv { left, right } => op(left, right, concrete::FDIV, env, ctx),
        Node::Mod { left, right } => op(left, right, concrete::MOD, env, ctx),
        Node::Pow { left, right } => op(left, right, concrete::POW, env, ctx),
        Node::Le { left, right } => op(left, right, concrete::LE, env, ctx),
        Node::Ge { left, right } => op(left, right, concrete::GE, env, ctx),
        Node::Leq { left, right } => op(left, right, concrete::LEQ, env, ctx),
        Node::Geq { left, right } => op(left, right, concrete::GEQ, env, ctx),

        Node::AnonFun { args, body } => {
            // TODO use Hindley Milner to infer type of arguments
            let mut arg_types: Vec<(String, (bool, ExpressionType))> = vec![];
            for arg in args {
                match &arg.node {
                    Node::IdType { id, mutable, _type } => {
                        let identifier = Identifier::try_from(id.deref())?;
                        let type_name = if let Some(_type) = _type {
                            TypeName::try_from(_type.deref())?
                        } else {
                            let msg = "Anonymous argument Must have type";
                            return Err(vec![TypeErr::new(&arg.pos, msg)]);
                        };

                        let expr_ty = ctx.lookup(&type_name, &arg.pos)?;
                        let matched = match_name(&identifier, &expr_ty, &env, &arg.pos)?;
                        for (id, (inner_mut, expr_ty)) in matched {
                            arg_types.push((id, (*mutable || inner_mut, expr_ty)));
                        }
                    }
                    _ => return Err(vec![TypeErr::new(&arg.pos, "Expected identifier")])
                }
            }

            let mut arg_env = env.clone();
            for (id, (mutable, expr_ty)) in arg_types.clone() {
                arg_env.insert(&id, mutable, &expr_ty);
            }

            let (ret_ty, _) = infer(body, &arg_env, ctx)?;
            let ret_ty = ret_ty.expr_ty(&body.pos)?;

            let actual_ty = ActualType::AnonFun {
                args:   arg_types.into_iter().map(|(_, (_, expr_ty))| expr_ty).collect(),
                ret_ty: Box::from(ret_ty)
            };
            let nullable_type = NullableType::new(false, &actual_ty);
            let expr_ty = ExpressionType::from(&nullable_type);

            Ok((InferType::from(&expr_ty), env.clone()))
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected operation")])
    }
}

fn unary_op(expr: &AST, overrides: &str, env: &Environment, ctx: &Context) -> InferResult {
    let (infer_type, env) = infer(expr, env, ctx)?;
    let fun = infer_type.expr_ty(&expr.pos)?.fun(overrides, &[], &expr.pos)?;
    match &fun.ty() {
        Some(fun_ty) => {
            let fun_ret_ty = ctx.lookup(fun_ty, &expr.pos)?;
            Ok((InferType::from(&fun_ret_ty).add_raises(&infer_type), env))
        }
        None => Ok((InferType::default().add_raises(&infer_type), env))
    }
}

fn op(left: &AST, right: &AST, overrides: &str, env: &Environment, ctx: &Context) -> InferResult {
    let (left_ty, left_env) = infer(left, env, ctx)?;
    let (right_ty, right_env) = infer(right, &left_env, ctx)?;

    let expr_ty = left_ty.expr_ty(&left.pos)?;
    let args = vec![TypeName::from(&right_ty.expr_ty(&right.pos)?)];
    let fun = expr_ty.fun(overrides, &args, &left.pos)?;

    match &fun.ty() {
        Some(fun_ty) => {
            let expr_ty = ctx.lookup(fun_ty, &left.pos.union(&right.pos))?;
            Ok((InferType::from(&expr_ty).add_raises(&left_ty).add_raises(&right_ty), right_env))
        }
        None => Ok((InferType::default().add_raises(&left_ty).add_raises(&right_ty), right_env))
    }
}
