use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::context::environment::Environment;
use crate::type_checker::context::field::Field;
use crate::type_checker::context::function::Function;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::context::{Context, ReturnType};
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

fn check_direct(ast: &AST, env: Environment, ctx: &Context) -> InferResult {
    match &ast.node {
        // TODO check imports (somewhere)
        Node::File { .. } => Ok((None, env)),
        Node::Import { .. } => Ok((None, env)),
        Node::FromImport { .. } => Ok((None, env)),

        // TODO create trait so fields and functions are interchangeable
        Node::Id { lit } => match env.lookup(lit) {
            Some(field) => Ok((Some(field.get_return_type_name()?), env)),
            None => Err(vec![TypeErr::new(&ast.pos, "Unrecognized variable")])
        },
        Node::IdType { .. } => Err(vec![TypeErr::new(&ast.pos, "Cannot inline type of variable")]),
        Node::VariableDef { .. } => {
            let new_field = Field::try_from(ast)?;
            Ok((None, env.add(&new_field, &ast.pos)?))
        }
        Node::FunDef { .. } => {
            let new_function = Function::try_from(ast)?;
            Ok((None, env.add_function(&new_function, &ast.pos)?))
        }

        // TODO create system for mutable types so we know that something can be reassigned to
        Node::Reassign { .. } => Ok((None, env)),

        Node::Class { .. } => Ok((None, env)),
        Node::Generic { .. } => Ok((None, env)),
        Node::Parent { .. } => Ok((None, env)),
        Node::Script { .. } => Ok((None, env)),
        Node::Init => Ok((None, env)),
        Node::AnonFun { .. } => Ok((None, env)),
        Node::Raises { .. } => Ok((None, env)),
        Node::Raise { .. } => Ok((None, env)),
        Node::Handle { .. } => Ok((None, env)),
        Node::Retry => Ok((None, env)),
        Node::With { .. } => Ok((None, env)),
        Node::FunctionCall { .. } => Ok((None, env)),
        Node::PropertyCall { .. } => Ok((None, env)),
        Node::TypeDef { .. } => Ok((None, env)),
        Node::TypeAlias { .. } => Ok((None, env)),
        Node::TypeTup { .. } => Ok((None, env)),
        Node::Type { .. } => Ok((None, env)),
        Node::TypeFun { .. } => Ok((None, env)),
        Node::Condition { .. } => Ok((None, env)),
        Node::FunArg { .. } => Ok((None, env)),
        Node::_Self => Ok((None, env)),
        Node::AddOp => Ok((None, env)),
        Node::SubOp => Ok((None, env)),
        Node::SqrtOp => Ok((None, env)),
        Node::MulOp => Ok((None, env)),
        Node::FDivOp => Ok((None, env)),
        Node::DivOp => Ok((None, env)),
        Node::PowOp => Ok((None, env)),
        Node::ModOp => Ok((None, env)),
        Node::EqOp => Ok((None, env)),
        Node::LeOp => Ok((None, env)),
        Node::GeOp => Ok((None, env)),
        Node::Set { .. } => Ok((None, env)),
        Node::SetBuilder { .. } => Ok((None, env)),
        Node::List { .. } => Ok((None, env)),
        Node::ListBuilder { .. } => Ok((None, env)),
        Node::Tuple { .. } => Ok((None, env)),
        Node::Range { .. } => Ok((None, env)),
        Node::Block { .. } => Ok((None, env)),
        Node::Real { .. } => Ok((None, env)),
        Node::Int { .. } => Ok((None, env)),
        Node::ENum { .. } => Ok((None, env)),
        Node::Str { .. } => Ok((None, env)),
        Node::Bool { .. } => Ok((None, env)),
        Node::AddU { .. } => Ok((None, env)),
        Node::Sub { .. } => Ok((None, env)),
        Node::SubU { .. } => Ok((None, env)),
        Node::FDiv { .. } => Ok((None, env)),
        Node::Mod { .. } => Ok((None, env)),
        Node::BAnd { .. } => Ok((None, env)),
        Node::BOr { .. } => Ok((None, env)),
        Node::BXOr { .. } => Ok((None, env)),
        Node::BOneCmpl { .. } => Ok((None, env)),
        Node::BLShift { .. } => Ok((None, env)),
        Node::BRShift { .. } => Ok((None, env)),
        Node::Sqrt { .. } => Ok((None, env)),

        Node::Add { left, right }
        | Node::Mul { left, right }
        | Node::Div { left, right }
        | Node::Pow { left, right }
        | Node::Le { left, right }
        | Node::Ge { left, right }
        | Node::Leq { left, right }
        | Node::Geq { left, right }
        | Node::Is { left, right }
        | Node::IsN { left, right }
        | Node::Eq { left, right }
        | Node::Neq { left, right }
        | Node::IsA { left, right }
        | Node::IsNA { left, right } => {
            let (left_type_name, left_env) = check_direct(left, env, ctx)?;
            let left_type_name =
                left_type_name.ok_or(TypeErr::new(&ast.pos, "Must be expression"))?;

            let left_type = ctx.lookup(&left_type_name)?;
            if left_type.overrides_op(&ast.node) {
                check_direct(right, env.union(&left_env), ctx)
            } else {
                Err(vec![TypeErr::new(&ast.pos, "Type does not override operator")])
            }
        }

        Node::And { left, right } | Node::Or { left, right } => {
            let (left_type, left_env) = check_direct(left, env, ctx)?;
            assert(&BOOLEAN, &left_type, &ast.pos)?;

            let (right_type, right_env) = check_direct(right, env.union(&left_env), ctx)?;
            assert(&BOOLEAN, &right_type, &ast.pos)?;

            Ok((right_type, right_env))
        }
        Node::Not { expr } => {
            let (expr_type, expr_env) = check_direct(expr, env, ctx)?;
            assert(&BOOLEAN, &expr_type, &ast.pos)?;
            Ok((expr_type, expr_env))
        }

        Node::IfElse { cond, then, _else } => {
            let (cond_type, cond_env) = check_direct(cond, env, ctx)?;
            assert(&BOOLEAN, &cond_type, &ast.pos)?;
            if let Some(_else) = _else {
                check_direct(_else, env.union(&cond_env), ctx)?;
            }
            check_direct(then, env.union(&cond_env), ctx)
        }

        // TODO create system for pattern matching
        Node::Match { .. } => Ok((None, env)),
        Node::Case { .. } => Ok((None, env)),

        Node::For { expr, body } => {
            let (range_type, range_env) = check_direct(expr, env, ctx)?;
            assert(&RANGE, &range_type, &ast.pos)?;
            check_direct(body, env.union(&range_env), ctx)
        }
        Node::In { left, right } => {
            let (_, left_env) = check_direct(left, env, ctx)?;
            // TODO create system for collections and checking their contained types
            check_direct(right, env.union(&left_env), ctx)
        }
        Node::Step { .. } => Ok((None, env)),
        Node::While { cond, body } => {
            let (cond_type, cond_env) = check_direct(cond, env, ctx)?;
            assert(&BOOLEAN, &cond_type, &ast.pos)?;
            check_direct(body, env.union(&cond_env), ctx)
        }
        Node::Break => Ok((None, env)),
        Node::Continue => Ok((None, env)),
        Node::Return { expr } => check_direct(expr, env, ctx),
        Node::ReturnEmpty => Ok((None, env)),
        Node::Underscore => Ok((None, env)),
        Node::Pass => Ok((None, env)),

        // TODO create system for optional types
        Node::Question { .. } => Ok((None, env)),

        Node::Print { .. } => Ok((None, env)),
        Node::Comment { .. } => Ok((None, env))
    }
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
