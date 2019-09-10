use std::any::Any;
use std::path::PathBuf;

use crate::common::position::Position;
use crate::parser::ast::ASTNode::TypeDef;
use crate::parser::ast::{ASTNode, ASTNodePos};
use crate::type_checker::context::class::Type;
use crate::type_checker::context::environment::Environment;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::context::{Context, ReturnType};
use crate::type_checker::type_result::{TypeErr, TypeResult};
use crate::type_checker::types::construct::RANGE;
use crate::type_checker::types::primitive::*;
use crate::type_checker::CheckInput;

pub type InferResult<T = Option<TypeName>> = std::result::Result<(T, Environment), Vec<TypeErr>>;

pub fn check(inputs: &[CheckInput], env: &Environment, ctx: &Context) -> Result<(), TypeErr> {
    for (input, src, path) in inputs {
        check_direct(input, env, ctx)
            .map_err(|errs| err.iter().map(|err| err.into_with_source(src, path)))?;
    }
    Ok(())
}

fn check_direct(node_pos: &ASTNodePos, env: &Environment, ctx: &Context) -> InferResult {
    Ok(match &node_pos.node {
        // TODO check imports (somewhere)
        ASTNode::File { .. } => None,
        ASTNode::Import { .. } => None,
        ASTNode::FromImport { .. } => None,

        ASTNode::Id { lit } =>
            env.lookup(lit).ok_or(TypeErr::new(&node_pos.position, "Unrecognized variable"))?,
        ASTNode::IdType { .. } =>
            Err(TypeErr::new(&node_pos.position, "Cannot inline type of variable")),
        var_def @ ASTNode::VariableDef { .. } => {
            let new_field = Field::try_from(var_def)?;
            Ok((None, env.add(new_field, &node_pos.position)?))
        }
        fun_def @ ASTNode::FunDef { .. } => {
            let new_function = Function::try_from(fun_def)?;
            Ok((None, env.add_function(new_function, &node_pos.position)?))
        }

        // TODO create system for mutable types so we know that something can be reassigned to
        ASTNode::Reassign { .. } => None,

        ASTNode::Class { .. } => None,
        ASTNode::Generic { .. } => None,
        ASTNode::Parent { .. } => None,
        ASTNode::Script { .. } => None,
        ASTNode::Init => None,
        ASTNode::AnonFun { .. } => None,
        ASTNode::Raises { .. } => None,
        ASTNode::Raise { .. } => None,
        ASTNode::Handle { .. } => None,
        ASTNode::Retry => None,
        ASTNode::With { .. } => None,
        ASTNode::FunctionCall { .. } => None,
        ASTNode::PropertyCall { .. } => None,
        ASTNode::TypeDef { .. } => None,
        ASTNode::TypeAlias { .. } => None,
        ASTNode::TypeTup { .. } => None,
        ASTNode::Type { .. } => None,
        ASTNode::TypeFun { .. } => None,
        ASTNode::Condition { .. } => None,
        ASTNode::FunArg { .. } => None,
        ASTNode::_Self => None,
        ASTNode::AddOp => None,
        ASTNode::SubOp => None,
        ASTNode::SqrtOp => None,
        ASTNode::MulOp => None,
        ASTNode::FDivOp => None,
        ASTNode::DivOp => None,
        ASTNode::PowOp => None,
        ASTNode::ModOp => None,
        ASTNode::EqOp => None,
        ASTNode::LeOp => None,
        ASTNode::GeOp => None,
        ASTNode::Set { .. } => None,
        ASTNode::SetBuilder { .. } => None,
        ASTNode::List { .. } => None,
        ASTNode::ListBuilder { .. } => None,
        ASTNode::Tuple { .. } => None,
        ASTNode::Range { .. } => None,
        ASTNode::Block { .. } => None,
        ASTNode::Real { .. } => None,
        ASTNode::Int { .. } => None,
        ASTNode::ENum { .. } => None,
        ASTNode::Str { .. } => None,
        ASTNode::Bool { .. } => None,
        ASTNode::AddU { .. } => None,
        ASTNode::Sub { .. } => None,
        ASTNode::SubU { .. } => None,
        ASTNode::FDiv { .. } => None,
        ASTNode::Mod { .. } => None,
        ASTNode::BAnd { .. } => None,
        ASTNode::BOr { .. } => None,
        ASTNode::BXOr { .. } => None,
        ASTNode::BOneCmpl { .. } => None,
        ASTNode::BLShift { .. } => None,
        ASTNode::BRShift { .. } => None,
        ASTNode::Sqrt { .. } => None,

        op @ ASTNode::Add { left, right }
        | op @ ASTNode::Mul { left, right }
        | op @ ASTNode::Div { left, right }
        | op @ ASTNode::Pow { left, right }
        | op @ ASTNode::Le { left, right }
        | op @ ASTNode::Ge { left, right }
        | op @ ASTNode::Leq { left, right }
        | op @ ASTNode::Geq { left, right }
        | op @ ASTNode::Is { left, right }
        | op @ ASTNode::IsN { left, right }
        | op @ ASTNode::Eq { left, right }
        | op @ ASTNode::Neq { left, right }
        | op @ ASTNode::IsA { left, right }
        | op @ ASTNode::IsNA { left, right } => {
            let (left_type_name, left_env) = check_direct(left, env, ctx)?;
            let left_type_name =
                left_type.ok_or(TypeErr::new(&node_pos.position, "Must be expression"))?;

            let left_type = ctx.lookup(left_type_name)?;
            left_type.overrides_op(op)?;

            check_direct(right, &env.union(&left_env), ctx)
        }

        ASTNode::And { left, right } | ASTNode::Or { left, right } => {
            let (left_type, left_env) = check_direct(left, env, ctx)?;
            assert(&BOOLEAN, &left_type, &node_pos.position)?;

            let (right_type, right_env) = check_direct(left, &env.union(&left_env), ctx)?;
            assert(&BOOLEAN, &right_type, &node_pos.position)?;

            Ok((right_type, right_env))
        }
        ASTNode::Not { expr } => {
            let (expr_type, expr_env) = check_direct(expr, env, ctx)?;
            assert(&BOOLEAN, &expr_type, &node_pos.position)?;
            Ok((expr_type, expr_env))
        }

        ASTNode::IfElse { cond, then, _else } => {
            let (cond_type, cond_env) = check_direct(expr, env, ctx)?;
            assert(&BOOLEAN, &range_type, &node_pos.position)?;
            if let Some(_else) = _else {
                check_direct(_else, &env.union(&cond_env), ctx)?;
            }
            check_direct(then, &env.union(&cond_env), ctx)
        }

        // TODO create system for pattern matching
        ASTNode::Match { .. } => None,
        ASTNode::Case { .. } => None,

        ASTNode::For { expr, body } => {
            let (range_type, range_env) = check_direct(expr, env, ctx)?;
            assert(&RANGE, &range_type, &node_pos.position)?;
            check_direct(body, &env.union(&range_env), ctx)
        }
        ASTNode::In { left, right } => {
            let (left_type, left_env) = check_direct(left, env, ctx)?;
            // TODO create system for collections and checking their contained types
            check_direct(right, env, ctx)
        }
        ASTNode::Step { .. } => None,
        ASTNode::While { cond, body } => {
            let (cond_type, cond_env) = check_direct(cond, env, ctx)?;
            assert(&BOOLEAN, &cond_type, &node_pos.position)?;
            check_direct(body, &env.union(&cond_env), ctx)
        }
        ASTNode::Break => None,
        ASTNode::Continue => None,
        ASTNode::Return { expr } => check_direct(expr, env, ctx),
        ASTNode::ReturnEmpty => None,
        ASTNode::Underscore => None,
        ASTNode::Pass => None,

        // TODO create system for optional types
        ASTNode::Question { .. } => None,

        ASTNode::Print { .. } => None,
        ASTNode::Comment { .. } => None
    })
}

fn assert(
    expected: &TypeName,
    actual: &Option<TypeName>,
    position: &Position
) -> Result<TypeName, TypeErr> {
    match actual {
        Some(actual) if actual == expected => Ok(actual.clone()),
        Some(actual) => Err(TypeErr::new(
            position,
            format!("Expected {} but was {}", expected, actual).as_str()
        )),
        None => Err(TypeErr::new(position, format!("Expected {}", expected).as_str()))
    }
}
