use crate::parser::ast::{Node, AST};
use crate::type_checker::context::Context;
use crate::type_checker::environment::infer_type::InferType;
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::assign::infer_assign;
use crate::type_checker::infer::bitwise_operation::infer_bitwise_op;
use crate::type_checker::infer::block::infer_block;
use crate::type_checker::infer::boolean_operation::infer_boolean_op;
use crate::type_checker::infer::call::infer_call;
use crate::type_checker::infer::class::infer_class;
use crate::type_checker::infer::collection::infer_coll;
use crate::type_checker::infer::control_flow::infer_control_flow;
use crate::type_checker::infer::error::infer_error;
use crate::type_checker::infer::literal::infer_literal;
use crate::type_checker::infer::operation::infer_op;
use crate::type_checker::infer::optional::infer_optional;
use crate::type_checker::type_result::TypeErr;
use crate::type_checker::CheckInput;

mod assign;
mod bitwise_operation;
mod block;
mod boolean_operation;
mod call;
mod class;
mod collection;
mod control_flow;
mod error;
mod literal;
mod operation;
mod optional;

pub type Inferred<T> = (T, Environment);
pub type InferResult<T = InferType> = std::result::Result<Inferred<T>, Vec<TypeErr>>;

pub fn infer_all(
    inputs: &[CheckInput],
    env: &Environment,
    ctx: &Context
) -> Result<(), Vec<TypeErr>> {
    let (_, errs): (Vec<_>, Vec<_>) = inputs
        .iter()
        .map(|(ast, source, path)| {
            infer(&Box::from(ast.clone()), &env.clone(), ctx, &State::new()).map_err(|errs| {
                errs.into_iter()
                    .map(|err| err.into_with_source(source, path))
                    .collect::<Vec<TypeErr>>()
            })
        })
        .partition(Result::is_ok);

    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs.into_iter().map(Result::unwrap_err).flatten().collect())
    }
}

fn infer(ast: &AST, env: &Environment, ctx: &Context, state: &State) -> InferResult {
    match &ast.node {
        // TODO analyse imports of File somewhere
        // TODO Check functions are pure if file is pure
        Node::File { modules, .. } => {
            let (_, errs): (Vec<_>, Vec<_>) = modules
                .iter()
                .map(|ast| infer(&Box::from(ast.clone()), env, ctx, state))
                .partition(Result::is_ok);

            if errs.is_empty() {
                Err(errs.into_iter().map(Result::unwrap_err).flatten().collect())
            } else {
                Ok((InferType::new(), env.clone()))
            }
        }
        Node::Import { .. } => Ok((InferType::new(), env.clone())),
        Node::FromImport { .. } => Ok((InferType::new(), env.clone())),

        Node::Init | Node::Class { .. } => infer_class(ast, env, ctx, state),
        Node::Generic { .. } | Node::Parent { .. } => infer_class(ast, env, ctx, state),

        Node::Script { .. } | Node::Block { .. } => infer_block(ast, env, ctx, state),

        Node::Id { .. } => infer_assign(ast, env, ctx, state),
        Node::Reassign { .. } => infer_assign(ast, env, ctx, state),
        Node::VariableDef { .. } => infer_assign(ast, env, ctx, state),
        Node::FunArg { .. } | Node::FunDef { .. } => infer_assign(ast, env, ctx, state),

        Node::Raises { .. } | Node::Raise { .. } => infer_error(ast, env, ctx, state),
        Node::Handle { .. } => infer_error(ast, env, ctx, state),
        Node::Retry => infer_error(ast, env, ctx, state),

        Node::With { .. } => unimplemented!(),
        Node::AnonFun { .. } => unimplemented!(),
        Node::FunctionCall { .. } | Node::PropertyCall { .. } => infer_call(ast, env, ctx, state),

        Node::IdType { .. } => Ok((InferType::new(), env.clone())),
        Node::TypeDef { .. } => Ok((InferType::new(), env.clone())),
        Node::TypeAlias { .. } => Ok((InferType::new(), env.clone())),
        Node::TypeTup { .. } => Ok((InferType::new(), env.clone())),
        Node::TypeUnion { .. } => Ok((InferType::new(), env.clone())),
        Node::Type { .. } => Ok((InferType::new(), env.clone())),
        Node::TypeFun { .. } => Ok((InferType::new(), env.clone())),

        Node::Condition { .. } => unimplemented!(),

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

        Node::Set { .. } | Node::SetBuilder { .. } => infer_coll(ast, env, ctx, state),
        Node::List { .. } | Node::ListBuilder { .. } => infer_coll(ast, env, ctx, state),
        Node::Tuple { .. } => infer_coll(ast, env, ctx, state),

        Node::Real { .. }
        | Node::Int { .. }
        | Node::ENum { .. }
        | Node::Str { .. }
        | Node::Bool { .. } => infer_literal(ast, env, ctx, state),

        Node::Add { .. } | Node::AddU { .. } => infer_op(ast, env, ctx, state),
        Node::Sub { .. } | Node::SubU { .. } => infer_op(ast, env, ctx, state),
        Node::Mul { .. } | Node::Div { .. } | Node::FDiv { .. } => infer_op(ast, env, ctx, state),
        Node::Mod { .. } => infer_op(ast, env, ctx, state),
        Node::Pow { .. } | Node::Sqrt { .. } => infer_op(ast, env, ctx, state),
        Node::Le { .. } | Node::Ge { .. } => infer_op(ast, env, ctx, state),
        Node::Leq { .. } | Node::Geq { .. } => infer_op(ast, env, ctx, state),

        Node::BAnd { .. } | Node::BOr { .. } | Node::BXOr { .. } =>
            infer_bitwise_op(ast, env, ctx, state),
        Node::BOneCmpl { .. } => infer_bitwise_op(ast, env, ctx, state),
        Node::BLShift { .. } | Node::BRShift { .. } => infer_bitwise_op(ast, env, ctx, state),

        Node::Is { .. } | Node::IsN { .. } => infer_boolean_op(ast, env, ctx, state),
        Node::IsA { .. } | Node::IsNA { .. } => infer_boolean_op(ast, env, ctx, state),
        Node::And { .. } | Node::Or { .. } => infer_boolean_op(ast, env, ctx, state),
        Node::Not { .. } => infer_boolean_op(ast, env, ctx, state),
        Node::Eq { .. } | Node::Neq { .. } => infer_boolean_op(ast, env, ctx, state),

        Node::IfElse { .. } => infer_control_flow(ast, env, ctx, state),
        Node::Match { .. } | Node::Case { .. } => infer_control_flow(ast, env, ctx, state),
        Node::For { .. } | Node::In { .. } | Node::Range { .. } | Node::Step { .. } =>
            infer_control_flow(ast, env, ctx, state),
        Node::While { .. } | Node::Break | Node::Continue =>
            infer_control_flow(ast, env, ctx, state),

        Node::Question { .. } => infer_optional(ast, env, ctx, state),

        Node::Return { expr } => infer(expr, env, ctx, state),
        Node::ReturnEmpty => Ok((InferType::new(), env.clone())),

        Node::Underscore => Ok((InferType::new(), env.clone())),
        Node::Pass => Ok((InferType::new(), env.clone())),
        Node::Print { .. } => Ok((InferType::new(), env.clone())),
        Node::Comment { .. } => Ok((InferType::new(), env.clone()))
    }
}
