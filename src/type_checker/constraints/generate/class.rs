use std::convert::TryFrom;
use std::ops::Deref;

use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::cons::Expect::{Expression, ExpressionAny, Type};
use crate::type_checker::constraints::cons::{Constraints, Expected};
use crate::type_checker::constraints::generate::definition::identifier_from_var;
use crate::type_checker::constraints::generate::ty::constrain_ty;
use crate::type_checker::constraints::generate::{gen_vec, generate};
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::type_name::TypeName;
use crate::type_checker::type_result::TypeErr;

pub fn gen_class(ast: &AST, env: &Environment, ctx: &Context, constr: &Constraints) -> Constrained {
    match &ast.node {
        Node::Class { body: Some(body), args, ty, .. } => match &body.node {
            Node::Block { statements } => {
                let (constr, env) = constrain_class_args(args, env, ctx, constr)?;
                let type_name = TypeName::try_from(ty.deref())?;
                let env = env.in_class_new(&Type { type_name });
                gen_vec(statements, &env, ctx, &constr)
            }
            _ => Err(vec![TypeErr::new(&body.pos, "Expected code block")])
        },
        Node::Class { .. } => Ok((constr.clone(), env.clone())),

        Node::TypeDef { body: Some(body), ty, .. } => {
            let type_name = TypeName::try_from(ty.deref())?;
            let env = env.in_class_new(&Type { type_name });
            generate(body, &env, ctx, constr)
        }
        Node::TypeDef { .. } => Ok((constr.clone(), env.clone())),

        Node::TypeAlias { conditions, ty, .. } => {
            let type_name = TypeName::try_from(ty.deref())?;
            let env = env.in_class_new(&Type { type_name });
            gen_vec(conditions, &env, ctx, constr)
        }
        Node::Condition { cond, el: Some(el) } => {
            let (constr, env) = generate(cond, env, ctx, constr)?;
            generate(el, &env, ctx, &constr)
        }
        Node::Condition { cond, .. } => generate(cond, env, ctx, constr),

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected class or type definition")])
    }
}

fn constrain_class_args(
    args: &Vec<AST>,
    env: &Environment,
    ctx: &Context,
    constr: &Constraints
) -> Constrained {
    let mut res = (constr.clone(), env.clone());
    for arg in args {
        match &arg.node {
            Node::FunArg { mutable, var, ty, default, .. } => {
                res = identifier_from_var(var, ty, *mutable, &res.0, &res.1)?;
                res = match (ty, default) {
                    (Some(ty), Some(default)) => constrain_ty(default, ty, &res.1, ctx, &res.0)?,
                    (None, Some(default)) => generate(default, &res.1, ctx, &res.0)?,
                    _ => res
                }
            }
            Node::VariableDef { mutable, var, ty, expression: Some(expr), .. } => {
                res = identifier_from_var(var, ty, *mutable, &res.0, &res.1)?;
                res = match ty {
                    Some(ty) => constrain_ty(expr, ty, &res.1, ctx, &res.0)?,
                    None => {
                        let left = Expected::new(&expr.pos, &Expression { ast: *expr.clone() });
                        res.0 = res.0.add(&left, &Expected::new(&expr.pos, &ExpressionAny));
                        generate(expr, &res.1, ctx, &res.0)?
                    }
                }
            }
            Node::VariableDef { mutable, var, ty, .. } =>
                res = identifier_from_var(var, ty, *mutable, &res.0, &res.1)?,
            _ => return Err(vec![TypeErr::new(&arg.pos, "Expected function argument")])
        }
    }

    Ok(res)
}
