use std::collections::HashSet;
use std::convert::TryFrom;
use std::ops::Deref;

use itertools::{EitherOrBoth, Itertools};

use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::cons::Constraints;
use crate::type_checker::constraints::cons::Expect;
use crate::type_checker::constraints::generate::call::generate_call;
use crate::type_checker::constraints::generate::common::gen_vec;
use crate::type_checker::constraints::generate::control_flow::gen_cntrl_flow;
use crate::type_checker::constraints::generate::operation::gen_operation;
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::function;
use crate::type_checker::context::function_arg::concrete::FunctionArg;
use crate::type_checker::context::ty;
use crate::type_checker::context::Context;
use crate::type_checker::environment::name::Identifier;
use crate::type_checker::environment::Environment;
use crate::type_checker::type_name::TypeName;
use crate::type_checker::type_result::TypeErr;

mod call;
mod control_flow;
mod operation;

mod common;

pub fn generate(ast: &AST, env: &Environment, ctx: &Context, constr: &Constraints) -> Constrained {
    match &ast.node {
        Node::File { modules, .. } => gen_vec(modules, env, ctx, constr),
        Node::Block { statements } => gen_vec(statements, env, ctx, constr),
        Node::Script { statements } => gen_vec(statements, env, ctx, constr),

        Node::Class { body, .. } =>
            if let Some(body) = body {
                match &body.node {
                    Node::Block { statements } => gen_vec(statements, env, ctx, constr),
                    _ => Err(vec![TypeErr::new(&body.pos, "Expected code block")])
                }
            } else {
                Ok((constr.clone(), env.clone()))
            },

        Node::TypeDef { body, .. } =>
            if let Some(body) = body {
                generate(body, env, ctx, constr)
            } else {
                Ok((constr.clone(), env.clone()))
            },
        Node::TypeAlias { conditions, .. } => gen_vec(conditions, env, ctx, constr),
        Node::Condition { cond, _else } => {
            let (constr, env) = generate(cond, env, ctx, constr)?;
            if let Some(el) = _else {
                generate(el, &env, ctx, &constr)
            } else {
                Ok((constr, env))
            }
        }

        Node::VariableDef { var, .. } => {
            // TODO check if indeed mutable
            let identifier = Identifier::try_from(var.deref())?;
            let mut env = env.clone();
            for (f_mut, f_name) in &identifier.fields() {
                env = env.insert_new(*f_mut, f_name);
            }
            Ok((constr.clone(), env))
        }
        Node::FunDef { fun_args, ret_ty, body, .. } => {
            for fun_arg in fun_args {
                match &fun_arg.node {
                    Node::FunArg { .. } => {}
                    _ => return Err(vec![TypeErr::new(&fun_arg.pos, "Expected function argument")])
                }
            }

            match (ret_ty, body) {
                (Some(ret_ty), Some(body)) => {
                    let type_name = TypeName::try_from(ret_ty.deref())?;
                    let constr = constr
                        .add(&Expect::Expression { ast: body.deref().clone() }, &Expect::Type {
                            type_name
                        });
                    generate(body, &env, ctx, &constr)
                }
                _ => Ok((constr.clone(), env.clone()))
            }
        }
        Node::Reassign { left, right } => {
            let constr = constr
                .add(&Expect::Expression { ast: left.deref().clone() }, &Expect::Expression {
                    ast: right.deref().clone()
                });
            let constr = constr
                .add(&Expect::Expression { ast: left.deref().clone() }, &Expect::Mutable {
                    expect: Box::from(Expect::AnyExpression)
                });
            let (constr, env) = generate(right, env, ctx, &constr)?;
            generate(left, &env, ctx, &constr)
        }

        Node::ConstructorCall { name, args } => generate_call(ast, env, ctx, constr),
        Node::FunctionCall { name, args } => generate_call(ast, env, ctx, constr),
        Node::PropertyCall { instance, property } => generate_call(ast, env, ctx, constr),

        Node::TypeTup { .. } => unimplemented!(),
        Node::TypeUnion { .. } => unimplemented!(),
        Node::Type { .. } => unimplemented!(),
        Node::TypeFun { .. } => unimplemented!(),

        Node::AnonFun { .. } => unimplemented!(),
        Node::Raises { .. } => unimplemented!(),
        Node::Raise { .. } => unimplemented!(),
        Node::Handle { .. } => gen_cntrl_flow(ast, env, ctx, constr),
        Node::With { .. } => unimplemented!(),
        Node::Id { lit } =>
            if env.lookup_new(lit, &ast.pos).is_ok() {
                Ok((constr.clone(), env.clone()))
            } else {
                Err(vec![TypeErr::new(&ast.pos, &format!("Unknown variable: {}", lit))])
            },

        Node::SetBuilder { .. } => unimplemented!(),
        Node::ListBuilder { .. } => unimplemented!(),
        Node::Set { elements } | Node::List { elements } =>
            if let Some(first) = elements.first() {
                let mut constr_env = (constr.clone(), env.clone());
                for element in elements {
                    constr_env.0 = constr_env
                        .0
                        .add(&Expect::Expression { ast: element.clone() }, &Expect::Expression {
                            ast: first.clone()
                        });
                    constr_env = generate(element, &env, &ctx, &constr)?;
                }
                Ok(constr_env)
            } else {
                Ok((constr.clone(), env.clone()))
            },
        Node::Tuple { elements } => {
            let mut constr_env = (constr.clone(), env.clone());
            for element in elements {
                constr_env = generate(element, &env, &ctx, &constr)?;
            }
            Ok(constr_env)
        }
        Node::Range { .. } => unimplemented!(),

        Node::Real { .. } => gen_operation(ast, env, ctx, constr),
        Node::Int { .. } => gen_operation(ast, env, ctx, constr),
        Node::ENum { .. } => gen_operation(ast, env, ctx, constr),
        Node::Str { .. } => gen_operation(ast, env, ctx, constr),
        Node::Bool { .. } => gen_operation(ast, env, ctx, constr),

        Node::Add { .. } => gen_operation(ast, env, ctx, constr),
        Node::Sub { .. } => gen_operation(ast, env, ctx, constr),
        Node::Mul { .. } => gen_operation(ast, env, ctx, constr),
        Node::Div { .. } => gen_operation(ast, env, ctx, constr),
        Node::FDiv { .. } => gen_operation(ast, env, ctx, constr),
        Node::Pow { .. } => gen_operation(ast, env, ctx, constr),
        Node::Le { .. } => gen_operation(ast, env, ctx, constr),
        Node::Ge { .. } => gen_operation(ast, env, ctx, constr),
        Node::Leq { .. } => gen_operation(ast, env, ctx, constr),
        Node::Geq { .. } => gen_operation(ast, env, ctx, constr),
        Node::Eq { .. } => gen_operation(ast, env, ctx, constr),
        Node::Mod { .. } => gen_operation(ast, env, ctx, constr),
        Node::Neq { .. } => gen_operation(ast, env, ctx, constr),
        Node::AddU { .. } | Node::SubU { .. } => gen_operation(ast, env, ctx, constr),
        Node::Sqrt { .. } => gen_operation(ast, env, ctx, constr),

        Node::BOneCmpl { .. } => gen_operation(ast, env, ctx, constr),
        Node::BAnd { .. } => gen_operation(ast, env, ctx, constr),
        Node::BOr { .. } | Node::BXOr { .. } => gen_operation(ast, env, ctx, constr),
        Node::BLShift { .. } | Node::BRShift { .. } => gen_operation(ast, env, ctx, constr),

        Node::Is { .. } | Node::IsN { .. } => gen_operation(ast, env, ctx, constr),
        Node::IsA { .. } | Node::IsNA { .. } => gen_operation(ast, env, ctx, constr),
        Node::Not { .. } | Node::And { .. } | Node::Or { .. } =>
            gen_operation(ast, env, ctx, constr),

        Node::IfElse { .. } => gen_cntrl_flow(ast, env, ctx, constr),
        Node::Case { .. } => gen_cntrl_flow(ast, env, ctx, constr),
        Node::Match { .. } => gen_cntrl_flow(ast, env, ctx, constr),
        Node::For { .. } => gen_cntrl_flow(ast, env, ctx, constr),
        Node::In { .. } => gen_cntrl_flow(ast, env, ctx, constr),
        Node::Step { .. } => gen_cntrl_flow(ast, env, ctx, constr),
        Node::While { .. } => gen_cntrl_flow(ast, env, ctx, constr),

        Node::Return { expr } => {
            let constr = constr
                .add(&Expect::Expression { ast: expr.deref().clone() }, &Expect::AnyExpression);
            generate(expr, env, ctx, &constr)
        }

        Node::Question { left, right } => {
            let constr = constr
                .add(&Expect::Expression { ast: left.deref().clone() }, &Expect::Nullable {
                    expect: Box::from(Expect::AnyExpression)
                });
            let constr = constr
                .add(&Expect::Expression { ast: right.deref().clone() }, &Expect::AnyExpression);
            let (constr, env) = generate(left, env, ctx, &constr)?;
            generate(right, &env, ctx, &constr)
        }
        Node::QuestionOp { expr } => {
            let constr = constr
                .add(&Expect::Expression { ast: expr.deref().clone() }, &Expect::AnyExpression);
            generate(expr, env, ctx, &constr)
        }

        Node::Print { expr } => {
            let constr = constr
                .add(&Expect::Expression { ast: expr.deref().clone() }, &Expect::AnyExpression);
            generate(expr, env, ctx, &constr)
        }

        _ => Ok((constr.clone(), env.clone()))
    }
}
