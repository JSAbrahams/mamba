use std::collections::HashSet;

use crate::parser::ast::{Node, AST};
use crate::type_checker::context::Context;
use crate::type_checker::environment::expression_type::ExpressionType;
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::assign::infer_assign;
use crate::type_checker::infer::bitwise_operation::infer_bitwise_operation;
use crate::type_checker::infer::block::infer_block;
use crate::type_checker::infer::boolean_operation::infer_boolean_operation;
use crate::type_checker::infer::collection::infer_collection;
use crate::type_checker::infer::error::infer_error;
use crate::type_checker::infer::operation::infer_operation;
use crate::type_checker::type_result::TypeErr;
use crate::type_checker::CheckInput;

mod assign;
mod bitwise_operation;
mod block;
mod boolean_operation;
mod collection;
mod control_flow;
mod error;
mod operation;

pub type Inferred<T> = (T, Environment, State);
pub type InferResult<T = Option<ExpressionType>> = std::result::Result<Inferred<T>, Vec<TypeErr>>;

pub fn infer_all(
    inputs: &[CheckInput],
    env: &Environment,
    ctx: &Context
) -> Result<(), Vec<TypeErr>> {
    let (_, errs): (Vec<_>, Vec<_>) = inputs
        .iter()
        .map(|input| infer(&Box::from(input.clone()), &env.clone(), ctx, &State::new()))
        .partition(Result::is_ok);

    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs.iter().map(Result::unwrap).flatten().collect())
    }
}

fn infer(ast: &Box<AST>, env: &Environment, ctx: &Context, state: &State) -> InferResult {
    match &ast.node {
        Node::File { pure, comments, imports, modules } => {
            let (_, errs): (Vec<_>, Vec<_>) = modules
                .iter()
                .map(|ast| infer(&Box::from(ast.clone()), env, ctx, state))
                .partition(Result::is_ok);

            if errs.is_empty() {
                Err(errs.into_iter().map(Result::unwrap_err).collect())
            } else {
                Ok((None, env.clone(), state.clone()))
            }
        }
        Node::Import { .. } => Ok((None, env.clone(), state.clone())),
        Node::FromImport { .. } => Ok((None, env.clone(), state.clone())),

        Node::Class { body, .. } => {
            infer(body, env, ctx, state)?;
            Ok((None, env.clone(), state.clone()))
        }
        Node::Generic { .. } => Ok((None, env.clone(), state.clone())),
        Node::Parent { .. } => Ok((None, env.clone(), state.clone())),
        Node::Script { statements } => {
            let ast = Box::from(AST {
                pos:  ast.pos.clone(),
                node: Node::Block { statements: statements.clone() }
            });
            infer(&ast, env, ctx, state)
        }
        Node::Init => Ok((None, env.clone(), state.clone())),

        Node::Id { lit } => unimplemented!(),
        Node::Reassign { .. } => infer_assign(ast, env, ctx, state),
        Node::VariableDef { .. } => infer_assign(ast, env, ctx, state),
        Node::FunArg { .. } | Node::FunDef { .. } => infer_assign(ast, env, ctx, state),

        Node::Raises { .. } | Node::Raise { .. } => infer_error(ast, env, ctx, state),
        Node::Handle { .. } => infer_error(ast, env, ctx, state),
        Node::Retry => infer_error(ast, env, ctx, state),

        Node::With { .. } => unimplemented!(),
        Node::AnonFun { .. } => unimplemented!(),
        Node::FunctionCall { .. } => unimplemented!(),
        Node::PropertyCall { .. } => unimplemented!(),

        Node::IdType { .. } => Ok((None, env.clone(), state.clone())),
        Node::TypeDef { .. } => Ok((None, env.clone(), state.clone())),
        Node::TypeAlias { .. } => Ok((None, env.clone(), state.clone())),
        Node::TypeTup { .. } => Ok((None, env.clone(), state.clone())),
        Node::Type { .. } => Ok((None, env.clone(), state.clone())),
        Node::TypeFun { .. } => Ok((None, env.clone(), state.clone())),

        Node::Condition { .. } => unimplemented!(),

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

        Node::Set { elements } | Node::SetBuilder { .. } => infer_collection(ast, env, ctx, state),
        Node::List { .. } | Node::ListBuilder { .. } => infer_collection(ast, env, ctx, state),
        Node::Tuple { .. } => infer_collection(ast, env, ctx, state),

        Node::Block { .. } => infer_block(ast, env, ctx, state),
        Node::Real { .. } => Ok((Some(ExpressionType::FLOAT), env.clone(), state.clone())),

        Node::Int { .. } => Ok((Some(ExpressionType::INT), env.clone(), state.clone())),
        Node::ENum { .. } => unimplemented!(),
        Node::Str { .. } => Ok((Some(ExpressionType::STRING), env.clone(), state.clone())),
        Node::Bool { .. } => Ok((Some(ExpressionType::BOOL), env.clone(), state.clone())),
        Node::Add { .. } | Node::AddU { .. } => infer_operation(ast, env, ctx, state),

        Node::Sub { .. } | Node::SubU { .. } => infer_operation(ast, env, ctx, state),
        Node::Mul { .. } | Node::Div { .. } | Node::FDiv { .. } =>
            infer_operation(ast, env, ctx, state),
        Node::Mod { .. } => infer_operation(ast, env, ctx, state),
        Node::Pow { .. } | Node::Sqrt { .. } => infer_operation(ast, env, ctx, state),
        Node::Le { .. } | Node::Ge { .. } => infer_operation(ast, env, ctx, state),
        Node::Leq { .. } | Node::Geq { .. } => infer_operation(ast, env, ctx, state),
        Node::Eq { .. } => infer_operation(ast, env, ctx, state),
        Node::BAnd { .. } | Node::BOr { .. } | Node::BXOr { .. } =>
            infer_bitwise_operation(ast, env, ctx, state),

        Node::BOneCmpl { .. } => infer_bitwise_operation(ast, env, ctx, state),
        Node::BLShift { .. } | Node::BRShift { .. } =>
            infer_bitwise_operation(ast, env, ctx, state),
        Node::Is { .. } => infer_boolean_operation(ast, env, ctx, state),

        Node::IsN { .. } => infer_boolean_operation(ast, env, ctx, state),
        Node::Neq { .. } => infer_boolean_operation(ast, env, ctx, state),
        Node::IsA { .. } => infer_boolean_operation(ast, env, ctx, state),
        Node::IsNA { .. } => infer_boolean_operation(ast, env, ctx, state),
        Node::Not { .. } => infer_boolean_operation(ast, env, ctx, state),
        Node::And { .. } => infer_boolean_operation(ast, env, ctx, state),
        Node::Or { .. } => infer_boolean_operation(ast, env, ctx, state),

        Node::IfElse { cond, then, _else } => infer_collection(ast, env, ctx, state),
        Node::Match { .. } | Node::Case { .. } => infer_collection(ast, env, ctx, state),
        Node::For { .. } | Node::In { .. } | Node::Range { .. } | Node::Step { .. } =>
            infer_collection(ast, env, ctx, state),
        Node::While { .. } | Node::Break | Node::Continue => infer_collection(ast, env, ctx, state),

        Node::Return { expr } => infer(expr, env, ctx, state),
        Node::ReturnEmpty => Ok((None, env.clone(), state.clone())),

        Node::Underscore => unimplemented!(),

        Node::Pass => Ok((None, env.clone(), state.clone())),
        Node::Question { left, right } =>
            match (infer(left, env, ctx, state)?, infer(right, env, ctx, state)?) {
                ((Some(left_ty), left_env), (Some(right_ty), right_env)) =>
                    if left_ty.nullable && left_ty == right_ty {
                        Ok((Some(left_ty), env.clone(), state.clone(())))
                    } else if left_ty == right_ty {
                        Err(vec![TypeErr::new(&ast.pos, "Must be nullable type")])
                    } else {
                        Err(vec![TypeErr::new(&ast.pos, "Must be equal types")])
                    },
                ((None, _), (..)) | ((..), (None, _)) =>
                    Err(vec![TypeErr::new(&ast.pos, "Must have type")]),
            },

        Node::Print { .. } => Ok((None, env.clone(), state.clone())),
        Node::Comment { .. } => Ok((None, env.clone(), state.clone()))
    }
}
