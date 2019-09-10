use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::context::environment::Environment;
use crate::type_checker::context::field::Field;
use crate::type_checker::context::function::Function;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::context::{Context, ReturnType};
use crate::type_checker::type_result::TypeErr;
use crate::type_checker::CheckInput;
use std::convert::TryFrom;

pub type InferResult<T = Option<TypeName>> = std::result::Result<(T, Environment), Vec<TypeErr>>;

pub fn check(inputs: &[CheckInput], env: &Environment, ctx: &Context) -> Result<(), Vec<TypeErr>> {
    for (input, ..) in inputs {
        check_direct(input, &env.clone(), ctx)?;
    }
    Ok(())
}

fn check_direct(ast: &AST, env: &Environment, ctx: &Context) -> InferResult {
    match &ast.node {
        // TODO check imports (somewhere)
        Node::File { .. } => Ok((None, env.clone())),
        Node::Import { .. } => Ok((None, env.clone())),
        Node::FromImport { .. } => Ok((None, env.clone())),

        // TODO create trait so fields and functions are interchangeable
        Node::Id { lit } => match env.lookup(lit) {
            Some(field) => Ok((Some(field.get_return_type_name()?), env.clone())),
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
        Node::Reassign { .. } => Ok((None, env.clone())),

        Node::Class { .. } => Ok((None, env.clone())),
        Node::Generic { .. } => Ok((None, env.clone())),
        Node::Parent { .. } => Ok((None, env.clone())),
        Node::Script { .. } => Ok((None, env.clone())),
        Node::Init => Ok((None, env.clone())),
        Node::AnonFun { .. } => Ok((None, env.clone())),
        Node::Raises { .. } => Ok((None, env.clone())),
        Node::Raise { .. } => Ok((None, env.clone())),
        Node::Handle { .. } => Ok((None, env.clone())),
        Node::Retry => Ok((None, env.clone())),
        Node::With { .. } => Ok((None, env.clone())),
        Node::FunctionCall { .. } => Ok((None, env.clone())),
        Node::PropertyCall { .. } => Ok((None, env.clone())),
        Node::TypeDef { .. } => Ok((None, env.clone())),
        Node::TypeAlias { .. } => Ok((None, env.clone())),
        Node::TypeTup { .. } => Ok((None, env.clone())),
        Node::Type { .. } => Ok((None, env.clone())),
        Node::TypeFun { .. } => Ok((None, env.clone())),
        Node::Condition { .. } => Ok((None, env.clone())),
        Node::FunArg { .. } => Ok((None, env.clone())),
        Node::_Self => Ok((None, env.clone())),
        Node::AddOp => Ok((None, env.clone())),
        Node::SubOp => Ok((None, env.clone())),
        Node::SqrtOp => Ok((None, env.clone())),
        Node::MulOp => Ok((None, env.clone())),
        Node::FDivOp => Ok((None, env.clone())),
        Node::DivOp => Ok((None, env.clone())),
        Node::PowOp => Ok((None, env.clone())),
        Node::ModOp => Ok((None, env.clone())),
        Node::EqOp => Ok((None, env.clone())),
        Node::LeOp => Ok((None, env.clone())),
        Node::GeOp => Ok((None, env.clone())),
        Node::Set { .. } => Ok((None, env.clone())),
        Node::SetBuilder { .. } => Ok((None, env.clone())),
        Node::List { .. } => Ok((None, env.clone())),
        Node::ListBuilder { .. } => Ok((None, env.clone())),
        Node::Tuple { .. } => Ok((None, env.clone())),
        Node::Range { .. } => Ok((None, env.clone())),
        Node::Block { .. } => Ok((None, env.clone())),
        Node::Real { .. } => Ok((None, env.clone())),
        Node::Int { .. } => Ok((None, env.clone())),
        Node::ENum { .. } => Ok((None, env.clone())),
        Node::Str { .. } => Ok((None, env.clone())),
        Node::Bool { .. } => Ok((None, env.clone())),
        Node::AddU { .. } => Ok((None, env.clone())),
        Node::Sub { .. } => Ok((None, env.clone())),
        Node::SubU { .. } => Ok((None, env.clone())),
        Node::FDiv { .. } => Ok((None, env.clone())),
        Node::Mod { .. } => Ok((None, env.clone())),
        Node::BAnd { .. } => Ok((None, env.clone())),
        Node::BOr { .. } => Ok((None, env.clone())),
        Node::BXOr { .. } => Ok((None, env.clone())),
        Node::BOneCmpl { .. } => Ok((None, env.clone())),
        Node::BLShift { .. } => Ok((None, env.clone())),
        Node::BRShift { .. } => Ok((None, env.clone())),
        Node::Sqrt { .. } => Ok((None, env.clone())),

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
                check_direct(right, &env.union(&left_env), ctx)
            } else {
                Err(vec![TypeErr::new(&ast.pos, "Type does not override operator")])
            }
        }

        Node::And { left, right } | Node::Or { left, right } => {
            let (left_type, left_env) = check_direct(left, env, ctx)?;
            //            assert(&BOOLEAN, &left_type, &ast.pos)?;

            let (right_type, right_env) = check_direct(right, &env.union(&left_env), ctx)?;
            //            assert(&BOOLEAN, &right_type, &ast.pos)?;

            Ok((right_type, right_env))
        }
        Node::Not { expr } => {
            let (expr_type, expr_env) = check_direct(expr, env, ctx)?;
            //            assert(&BOOLEAN, &expr_type, &ast.pos)?;
            Ok((expr_type, expr_env))
        }

        Node::IfElse { cond, then, _else } => {
            let (cond_type, cond_env) = check_direct(cond, env, ctx)?;
            //            assert(&BOOLEAN, &cond_type, &ast.pos)?;
            if let Some(_else) = _else {
                check_direct(_else, &env.union(&cond_env), ctx)?;
            }
            check_direct(then, &env.union(&cond_env), ctx)
        }

        // TODO create system for pattern matching
        Node::Match { .. } => Ok((None, env.clone())),
        Node::Case { .. } => Ok((None, env.clone())),

        Node::For { expr, body } => {
            let (range_type, range_env) = check_direct(expr, env, ctx)?;
            //            assert(&RANGE, &range_type, &ast.pos)?;
            check_direct(body, &env.union(&range_env), ctx)
        }
        Node::In { left, right } => {
            let (_, left_env) = check_direct(left, env, ctx)?;
            // TODO create system for collections and checking their contained types
            check_direct(right, &env.union(&left_env), ctx)
        }
        Node::Step { .. } => Ok((None, env.clone())),
        Node::While { cond, body } => {
            let (cond_type, cond_env) = check_direct(cond, env, ctx)?;
            //            assert(Primitive.int(), &cond_type, &ast.pos)?;
            check_direct(body, &env.union(&cond_env), ctx)
        }
        Node::Break => Ok((None, env.clone())),
        Node::Continue => Ok((None, env.clone())),
        Node::Return { expr } => check_direct(expr, env, ctx),
        Node::ReturnEmpty => Ok((None, env.clone())),
        Node::Underscore => Ok((None, env.clone())),
        Node::Pass => Ok((None, env.clone())),

        // TODO create system for optional types
        Node::Question { .. } => Ok((None, env.clone())),

        Node::Print { .. } => Ok((None, env.clone())),
        Node::Comment { .. } => Ok((None, env.clone()))
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
