use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::context::environment::Environment;
use crate::type_checker::context::field::Field;
use crate::type_checker::context::function::Function;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::context::Context;
use crate::type_checker::type_result::TypeErr;
use crate::type_checker::types::construct::RANGE;
use crate::type_checker::types::primitive::*;
use crate::type_checker::CheckInput;
use std::convert::TryFrom;

pub type InferResult<T = Option<TypeName>> = std::result::Result<(T, Environment), Vec<TypeErr>>;

pub fn check(inputs: &[CheckInput], env: Environment, ctx: &Context) -> Result<(), Vec<TypeErr>> {
    for (input, src, path) in inputs {
        check_direct(input, env, ctx).map_err(|errs| {
            errs.iter().map(|err| err.into_with_source(src, path)).collect::<Vec<TypeErr>>()
        })?;
    }
    Ok(())
}

fn check_direct(node_pos: &AST, env: Environment, ctx: &Context) -> InferResult {
    Ok(match &node_pos.node {
        // TODO check imports (somewhere)
        Node::File { .. } => (None, env),
        Node::Import { .. } => (None, env),
        Node::FromImport { .. } => (None, env),

        Node::Id { lit } =>
            Ok((env.lookup(lit).ok_or(TypeErr::new(&node_pos.pos, "Unrecognized variable"))?, env)),
        Node::IdType { .. } => Err(TypeErr::new(&node_pos.pos, "Cannot inline type of variable")),
        Node::VariableDef { .. } => {
            let new_field = Field::try_from(node_pos)?;
            Ok((None, env.add(&new_field, &node_pos.pos)?))
        }
        Node::FunDef { .. } => {
            let new_function = Function::try_from(node_pos)?;
            Ok((None, env.add_function(&new_function, &node_pos.pos)?))
        }

        // TODO create system for mutable types so we know that something can be reassigned to
        Node::Reassign { .. } => (None, env),

        Node::Class { .. } => (None, env),
        Node::Generic { .. } => (None, env),
        Node::Parent { .. } => (None, env),
        Node::Script { .. } => (None, env),
        Node::Init => (None, env),
        Node::AnonFun { .. } => (None, env),
        Node::Raises { .. } => (None, env),
        Node::Raise { .. } => (None, env),
        Node::Handle { .. } => (None, env),
        Node::Retry => (None, env),
        Node::With { .. } => (None, env),
        Node::FunctionCall { .. } => (None, env),
        Node::PropertyCall { .. } => (None, env),
        Node::TypeDef { .. } => (None, env),
        Node::TypeAlias { .. } => (None, env),
        Node::TypeTup { .. } => (None, env),
        Node::Type { .. } => (None, env),
        Node::TypeFun { .. } => (None, env),
        Node::Condition { .. } => (None, env),
        Node::FunArg { .. } => (None, env),
        Node::_Self => (None, env),
        Node::AddOp => (None, env),
        Node::SubOp => (None, env),
        Node::SqrtOp => (None, env),
        Node::MulOp => (None, env),
        Node::FDivOp => (None, env),
        Node::DivOp => (None, env),
        Node::PowOp => (None, env),
        Node::ModOp => (None, env),
        Node::EqOp => (None, env),
        Node::LeOp => (None, env),
        Node::GeOp => (None, env),
        Node::Set { .. } => (None, env),
        Node::SetBuilder { .. } => (None, env),
        Node::List { .. } => (None, env),
        Node::ListBuilder { .. } => (None, env),
        Node::Tuple { .. } => (None, env),
        Node::Range { .. } => (None, env),
        Node::Block { .. } => (None, env),
        Node::Real { .. } => (None, env),
        Node::Int { .. } => (None, env),
        Node::ENum { .. } => (None, env),
        Node::Str { .. } => (None, env),
        Node::Bool { .. } => (None, env),
        Node::AddU { .. } => (None, env),
        Node::Sub { .. } => (None, env),
        Node::SubU { .. } => (None, env),
        Node::FDiv { .. } => (None, env),
        Node::Mod { .. } => (None, env),
        Node::BAnd { .. } => (None, env),
        Node::BOr { .. } => (None, env),
        Node::BXOr { .. } => (None, env),
        Node::BOneCmpl { .. } => (None, env),
        Node::BLShift { .. } => (None, env),
        Node::BRShift { .. } => (None, env),
        Node::Sqrt { .. } => (None, env),

        op @ Node::Add { left, right }
        | op @ Node::Mul { left, right }
        | op @ Node::Div { left, right }
        | op @ Node::Pow { left, right }
        | op @ Node::Le { left, right }
        | op @ Node::Ge { left, right }
        | op @ Node::Leq { left, right }
        | op @ Node::Geq { left, right }
        | op @ Node::Is { left, right }
        | op @ Node::IsN { left, right }
        | op @ Node::Eq { left, right }
        | op @ Node::Neq { left, right }
        | op @ Node::IsA { left, right }
        | op @ Node::IsNA { left, right } => {
            let (left_type_name, left_env) = check_direct(left, env, ctx)?;
            let left_type_name =
                left_type_name.ok_or(TypeErr::new(&node_pos.pos, "Must be expression"))?;

            let left_type = ctx.lookup(left_type_name)?;
            left_type.overrides_op(op)?;

            check_direct(right, env.union(&left_env), ctx)
        }

        Node::And { left, right } | Node::Or { left, right } => {
            let (left_type, left_env) = check_direct(left, env, ctx)?;
            assert(&BOOLEAN, &left_type, &node_pos.pos)?;

            let (right_type, right_env) = check_direct(left, env.union(&left_env), ctx)?;
            assert(&BOOLEAN, &right_type, &node_pos.pos)?;

            Ok((right_type, right_env))
        }
        Node::Not { expr } => {
            let (expr_type, expr_env) = check_direct(expr, env, ctx)?;
            assert(&BOOLEAN, &expr_type, &node_pos.pos)?;
            Ok((expr_type, expr_env))
        }

        Node::IfElse { cond, then, _else } => {
            let (cond_type, cond_env) = check_direct(cond, env, ctx)?;
            assert(&BOOLEAN, &cond_type, &node_pos.pos)?;
            if let Some(_else) = _else {
                check_direct(_else, env.union(&cond_env), ctx)?;
            }
            check_direct(then, env.union(&cond_env), ctx)
        }

        // TODO create system for pattern matching
        Node::Match { .. } => None,
        Node::Case { .. } => None,

        Node::For { expr, body } => {
            let (range_type, range_env) = check_direct(expr, env, ctx)?;
            assert(&RANGE, &range_type, &node_pos.pos)?;
            check_direct(body, env.union(&range_env), ctx)
        }
        Node::In { left, right } => {
            let (left_type, left_env) = check_direct(left, env, ctx)?;
            // TODO create system for collections and checking their contained types
            check_direct(right, env, ctx)
        }
        Node::Step { .. } => None,
        Node::While { cond, body } => {
            let (cond_type, cond_env) = check_direct(cond, env, ctx)?;
            assert(&BOOLEAN, &cond_type, &node_pos.pos)?;
            check_direct(body, env.union(&cond_env), ctx)
        }
        Node::Break => None,
        Node::Continue => None,
        Node::Return { expr } => check_direct(expr, env, ctx),
        Node::ReturnEmpty => None,
        Node::Underscore => None,
        Node::Pass => None,

        // TODO create system for optional types
        Node::Question { .. } => None,

        Node::Print { .. } => None,
        Node::Comment { .. } => None
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
