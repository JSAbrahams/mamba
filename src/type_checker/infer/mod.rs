use crate::parser::ast::{Node, AST};
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::context::{function_arg, Context};
use crate::type_checker::environment::infer_type::InferType;
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

// TODO switch to system using restraint programming for more advanced type
// inference
pub fn infer_all(inputs: &[CheckInput], ctx: &Context) -> Result<(), Vec<TypeErr>> {
    let env = Environment::new();
    let (_, errs): (Vec<_>, Vec<_>) = inputs
        .iter()
        .map(|(ast, source, path)| {
            infer(&Box::from(ast.clone()), &env.clone(), ctx).map_err(|errs| {
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

fn infer(ast: &AST, env: &Environment, ctx: &Context) -> InferResult {
    match &ast.node {
        // TODO analyse imports of File somewhere
        // TODO Check functions are pure if file is pure
        Node::File { modules, .. } => {
            let (_, errs): (Vec<_>, Vec<_>) = modules
                .iter()
                .map(|ast| infer(&Box::from(ast.clone()), env, ctx))
                .partition(Result::is_ok);

            if errs.is_empty() {
                Ok((InferType::new(), env.clone()))
            } else {
                Err(errs.into_iter().map(Result::unwrap_err).flatten().collect())
            }
        }
        Node::Import { .. } => Ok((InferType::new(), env.clone())),
        Node::FromImport { .. } => Ok((InferType::new(), env.clone())),

        Node::Init | Node::Class { .. } => infer_class(ast, env, ctx),
        Node::Generic { .. } | Node::Parent { .. } => infer_class(ast, env, ctx),

        Node::Script { .. } | Node::Block { .. } => infer_block(ast, env, ctx),

        Node::Undefined => infer_assign(ast, env, ctx),
        Node::Id { .. } => infer_assign(ast, env, ctx),
        Node::Reassign { .. } => infer_assign(ast, env, ctx),
        Node::VariableDef { .. } => infer_assign(ast, env, ctx),
        Node::FunArg { .. } | Node::FunDef { .. } => infer_assign(ast, env, ctx),

        Node::Raises { .. } | Node::Raise { .. } => infer_error(ast, env, ctx),
        Node::Handle { .. } => infer_control_flow(ast, env, ctx),
        Node::Retry => infer_error(ast, env, ctx),

        Node::With { .. } => infer_error(ast, env, ctx),
        Node::AnonFun { .. } => infer_op(ast, env, ctx),
        Node::FunctionCall { .. } | Node::PropertyCall { .. } => infer_call(ast, env, ctx),

        Node::IdType { .. } => infer_assign(ast, env, ctx),
        Node::TypeDef { isa, body, .. } => {
            if let Some(isa) = isa {
                infer(isa, env, ctx)?;
            }
            if let Some(body) = body {
                infer(body, env, ctx)?;
            }
            Ok((InferType::new(), env.clone()))
        }
        Node::TypeAlias { isa, conditions, .. } => {
            infer(isa, env, ctx)?;
            for condition in conditions {
                infer(condition, env, ctx)?;
            }
            Ok((InferType::new(), env.clone()))
        }
        Node::TypeTup { types } => {
            for ty in types {
                infer(ty, env, ctx)?;
            }
            Ok((InferType::new(), env.clone()))
        }
        Node::TypeUnion { types } => {
            for ty in types {
                infer(ty, env, ctx)?;
            }
            Ok((InferType::new(), env.clone()))
        }
        Node::Type { id, generics } => {
            let id = match &id.node {
                Node::Id { lit } => lit.clone(),
                _ => return Err(vec![TypeErr::new(&id.pos, "Expected identifier")])
            };

            // TODO do something with generics
            ctx.lookup(&TypeName::from(id.as_str()), &ast.pos)?;
            for generic in generics {
                infer(generic, env, ctx)?;
            }
            Ok((InferType::new(), env.clone()))
        }
        Node::TypeFun { args, ret_ty } => {
            for arg in args {
                infer(arg, env, ctx)?;
            }
            infer(ret_ty, env, ctx)?;
            Ok((InferType::new(), env.clone()))
        }
        Node::QuestionOp { expr } => {
            infer(expr, env, ctx)?;
            Ok((InferType::new(), env.clone()))
        }

        Node::Condition { cond, _else } => {
            infer(cond, env, ctx)?;
            if let Some(_else) = _else {
                infer(_else, env, ctx)?;
            }
            Ok((InferType::new(), env.clone()))
        }

        Node::_Self =>
            Ok((InferType::from(&env.lookup(function_arg::concrete::SELF, &ast.pos)?), env.clone())),
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

        Node::Set { .. } | Node::SetBuilder { .. } => infer_coll(ast, env, ctx),
        Node::List { .. } | Node::ListBuilder { .. } => infer_coll(ast, env, ctx),
        Node::Tuple { .. } => infer_coll(ast, env, ctx),
        Node::In { .. } => infer_coll(ast, env, ctx),

        Node::Real { .. }
        | Node::Int { .. }
        | Node::ENum { .. }
        | Node::Str { .. }
        | Node::Bool { .. } => infer_literal(ast, env, ctx),

        Node::Add { .. } | Node::AddU { .. } => infer_op(ast, env, ctx),
        Node::Sub { .. } | Node::SubU { .. } => infer_op(ast, env, ctx),
        Node::Mul { .. } | Node::Div { .. } | Node::FDiv { .. } => infer_op(ast, env, ctx),
        Node::Mod { .. } => infer_op(ast, env, ctx),
        Node::Pow { .. } | Node::Sqrt { .. } => infer_op(ast, env, ctx),
        Node::Le { .. } | Node::Ge { .. } => infer_op(ast, env, ctx),
        Node::Leq { .. } | Node::Geq { .. } => infer_op(ast, env, ctx),

        Node::BAnd { .. } | Node::BOr { .. } | Node::BXOr { .. } => infer_bitwise_op(ast, env, ctx),
        Node::BOneCmpl { .. } => infer_bitwise_op(ast, env, ctx),
        Node::BLShift { .. } | Node::BRShift { .. } => infer_bitwise_op(ast, env, ctx),

        Node::Is { .. } | Node::IsN { .. } => infer_boolean_op(ast, env, ctx),
        Node::IsA { .. } | Node::IsNA { .. } => infer_boolean_op(ast, env, ctx),
        Node::And { .. } | Node::Or { .. } => infer_boolean_op(ast, env, ctx),
        Node::Not { .. } => infer_boolean_op(ast, env, ctx),
        Node::Eq { .. } | Node::Neq { .. } => infer_boolean_op(ast, env, ctx),

        Node::IfElse { .. } => infer_control_flow(ast, env, ctx),
        Node::Match { .. } | Node::Case { .. } => infer_control_flow(ast, env, ctx),
        Node::For { .. } | Node::Range { .. } | Node::Step { .. } =>
            infer_control_flow(ast, env, ctx),
        Node::While { .. } | Node::Break | Node::Continue => infer_control_flow(ast, env, ctx),

        Node::Question { .. } => infer_optional(ast, env, ctx),

        Node::Return { expr } =>
            if env.state.in_function {
                infer(expr, env, ctx)
            } else {
                Err(vec![TypeErr::new(&ast.pos, "Cannot have return outside function")])
            },
        Node::ReturnEmpty => Ok((InferType::new(), env.clone())),

        Node::Underscore => Ok((InferType::new(), env.clone())),
        Node::Pass => Ok((InferType::new(), env.clone())),
        Node::Print { expr } => {
            let (_, env) = infer(expr, env, ctx)?;
            Ok((InferType::new(), env))
        }
        Node::Comment { .. } => Ok((InferType::new(), env.clone()))
    }
}
