use std::collections::HashSet;

use crate::parser::ast::{Node, AST};
use crate::type_checker::context::Context;
use crate::type_checker::environment::function::Function;
use crate::type_checker::environment::ty::Type;
use crate::type_checker::environment::type_name::TypeName;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::state::State;
use crate::type_checker::infer::state::StateType::InLoop;
use crate::type_checker::ty::ExpressionType;
use crate::type_checker::type_result::TypeErr;
use crate::type_checker::CheckInput;

mod state;

pub type InferResult<T = Option<ExpressionType>> =
    std::result::Result<(T, Environment, State), Vec<TypeErr>>;

pub fn check(inputs: &[CheckInput], env: &Environment, ctx: &Context) -> Result<(), Vec<TypeErr>> {
    for (input, ..) in inputs {
        check_direct(&Box::from(input.clone()), &env.clone(), ctx, &State::new())?;
    }
    Ok(())
}

fn check_direct(ast: &Box<AST>, env: &Environment, ctx: &Context, state: &State) -> InferResult {
    match &ast.node {
        Node::File { pure, comments, imports, modules } => {
            let (_, errs): (Vec<_>, Vec<_>) = modules
                .iter()
                .map(|ast| check_direct(&Box::from(ast.clone()), env, ctx, state))
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
            check_direct(body, env, ctx, state)?;
            Ok((None, env.clone(), state.clone()))
        }
        Node::Generic { .. } => Ok((None, env.clone(), state.clone())),
        Node::Parent { .. } => Ok((None, env.clone(), state.clone())),
        Node::Script { statements } => check_direct(
            &Box::from(AST {
                pos:  ast.pos.clone(),
                node: Node::Block { statements: statements.clone() }
            }),
            env,
            ctx,
            state
        ),
        Node::Init => Ok((None, env.clone(), state.clone())),

        Node::Reassign { .. } => unimplemented!(),
        // TODO use forward and private, and get rid of ofmut
        Node::VariableDef { id_maybe_type, expression, .. } => match id_maybe_type.node {
            Node::IdType { mutable, _type, .. } => match (_type, expression) {
                (Some(ty), Some(expr)) => unimplemented!(),
                (None, Some(expr)) => unimplemented!(),
                (Some(ty), None) => unimplemented!(),
                (None, None) => unimplemented!()
            },
            _ => Err(vec![TypeErr::new(&ast.pos, "Expected identifier")])
        },
        Node::FunArg { .. } => unimplemented!(),
        Node::FunDef { fun_args, ret_ty, raises, body, .. } => unimplemented!(),

        Node::AnonFun { .. } => unimplemented!(),
        Node::Raises { .. } => unimplemented!(),
        Node::Raise { .. } => unimplemented!(),
        Node::Handle { expr_or_stmt, cases } => {
            if let (Some(expr_type), expr_env) = check_direct(expr_or_stmt, env, ctx, state)? {
                let state = state.clone().unhandled(&expr_type.raises);
                unimplemented!()
            } else {
                Err(vec![TypeErr::new(&ast.pos, "Expected expression")])
            }
        }
        Node::Retry => unimplemented!(),
        Node::With { .. } => unimplemented!(),
        Node::FunctionCall { .. } => unimplemented!(),
        Node::PropertyCall { .. } => unimplemented!(),
        Node::Id { lit } => unimplemented!(),

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

        Node::Set { elements } => unimplemented!(),
        Node::SetBuilder { .. } => unimplemented!(),
        Node::List { .. } => unimplemented!(),
        Node::ListBuilder { .. } => unimplemented!(),
        Node::Tuple { .. } => unimplemented!(),

        Node::Range { .. } => unimplemented!(),
        Node::Block { statements } => {
            let mut types = vec![];
            let mut env = env;
            let mut state = state;
            for statement in statements {
                let (statement_type, new_env, new_state) =
                    check_direct(&Box::from(statement.clone()), env, ctx, state);
                types.push(statement_type);
                env = new_env;
                state = new_state;
            }

            Ok((types.last(), env.clone()))
        }

        Node::Real { .. } => Ok((Some(ExpressionType::FLOAT), env.clone(), state.clone())),
        Node::Int { .. } => Ok((Some(ExpressionType::INT), env.clone(), state.clone())),
        Node::ENum { .. } => unimplemented!(),
        Node::Str { .. } => Ok((Some(ExpressionType::STRING), env.clone(), state.clone())),
        Node::Bool { .. } => Ok((Some(ExpressionType::BOOL), env.clone(), state.clone())),

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

        Node::BAnd { .. } => unimplemented!(),
        Node::BOr { .. } => unimplemented!(),
        Node::BXOr { .. } => unimplemented!(),
        Node::BOneCmpl { .. } => unimplemented!(),
        Node::BLShift { .. } => unimplemented!(),
        Node::BRShift { .. } => unimplemented!(),

        Node::Is { .. } => unimplemented!(),
        Node::IsN { .. } => unimplemented!(),
        Node::Neq { .. } => unimplemented!(),
        Node::IsA { .. } => unimplemented!(),
        Node::IsNA { .. } => unimplemented!(),
        Node::Not { .. } => unimplemented!(),
        Node::And { .. } => unimplemented!(),
        Node::Or { .. } => unimplemented!(),

        Node::IfElse { cond, then, _else } => {
            let (cond_type, cond_env) = check_direct(cond, env, ctx, state)?;
            let (then_type, then_env) = check_direct(then, &cond_env, ctx, state)?;

            if let Some(_else) = _else {
                let (else_type, else_env) = check_direct(_else, &cond_env, ctx, state)?;
                Ok((then_type, then_env.intersection(else_env), state.clone()))
            } else {
                Ok((then_type, then_env, state.clone()))
            }
        }
        Node::Match { .. } => unimplemented!(),
        Node::Case { .. } => unimplemented!(),
        Node::For { .. } => unimplemented!(),
        Node::In { .. } => unimplemented!(),
        Node::Step { .. } => unimplemented!(),
        Node::While { cond, body } => {
            let (cond_type, cond_env) = check_direct(cond, env, ctx, state)?;
            let (_, body_env) = check_direct(body, &cond_env, ctx, &state.is(InLoop)?)?;
            Ok((None, env.intersection(body_env), state.clone()))
        }
        Node::Break =>
            if state.in_loop {
                Ok((None, env.clone(), state.clone()))
            } else {
                Err(vec![TypeErr::new(&ast.pos, "Cannot occur outside loop")])
            },
        Node::Continue => Ok((None, env.clone(), state.clone())),

        Node::Return { expr } => check_direct(expr, env, ctx, state),
        Node::ReturnEmpty => Ok((None, env.clone(), state.clone())),

        Node::Underscore => unimplemented!(),

        Node::Pass => Ok((None, env.clone(), state.clone())),
        Node::Question { left, right } =>
            match (check_direct(left, env, ctx, state)?, check_direct(right, env, ctx, state)?) {
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

fn overrides(
    left: &Box<AST>,
    right: &Box<AST>,
    env: &Environment,
    ctx: &Context,
    state: &State,
    overrides: &str
) -> InferResult {
    let (left_expr_ty, left_env) = check_direct(left, env, ctx, state)?;
    let (right_expr_ty, right_env) = check_direct(right, &left_env, ctx, state)?;

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
