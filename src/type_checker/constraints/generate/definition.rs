use std::convert::TryFrom;
use std::ops::Deref;

use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::cons::Constraints;
use crate::type_checker::constraints::cons::Expect::{Expression, ExpressionAny};
use crate::type_checker::constraints::generate::generate;
use crate::type_checker::constraints::generate::resources::constrain_raises;
use crate::type_checker::constraints::generate::ty::constrain_ty;
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::function_arg::concrete::SELF;
use crate::type_checker::context::Context;
use crate::type_checker::environment::name::Identifier;
use crate::type_checker::environment::Environment;
use crate::type_checker::type_result::TypeErr;

pub fn gen_def(ast: &AST, env: &Environment, ctx: &Context, constr: &Constraints) -> Constrained {
    match &ast.node {
        Node::FunDef { fun_args, ret_ty, body, raises, .. } => {
            let (constr, env) = constrain_args(fun_args, env, ctx, constr)?;
            match (ret_ty, body) {
                (Some(ret_ty), Some(body)) => {
                    let (constr, env) = constrain_raises(body, raises, &env, ctx, &constr)?;
                    constrain_ty(body, ret_ty, &env, ctx, &constr)
                }
                (Some(ret_ty), None) => generate(ret_ty, &env, ctx, &constr),
                (None, Some(body)) => generate(body, &env, ctx, &constr),
                _ => Ok((constr.clone(), env.clone()))
            }
        }
        Node::FunArg { .. } =>
            Err(vec![TypeErr::new(&ast.pos, "Function argument cannot be top level")]),

        Node::VariableDef { mutable, var, ty, expression: Some(expr), .. } => {
            let env = identifier_from_var(var, *mutable, env)?;
            match ty {
                Some(ty) => constrain_ty(expr, ty, &env, ctx, constr),
                None => {
                    let constr = constr.add(&Expression { ast: *expr.clone() }, &ExpressionAny);
                    generate(expr, &env, ctx, &constr)
                }
            }
        }
        Node::VariableDef { mutable, var, ty, .. } => {
            let env = identifier_from_var(var, *mutable, env)?;
            match ty {
                Some(ty) => generate(ty, &env, ctx, constr),
                _ => Ok((constr.clone(), env))
            }
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected definition")])
    }
}

pub fn constrain_args(
    args: &Vec<AST>,
    env: &Environment,
    ctx: &Context,
    constr: &Constraints
) -> Constrained {
    let mut res = (constr.clone(), env.clone());
    for arg in args {
        match &arg.node {
            Node::FunArg { mutable, var, ty, default, .. } => {
                res = generate(var, &env, ctx, &constr)?;
                if &var.node == &Node::_Self {
                    let self_type = &env.state.in_class_new.clone().ok_or_else(|| {
                        TypeErr::new(&var.pos, &format!("{} cannot be outside class", SELF))
                    })?;
                    if default.is_some() {
                        return Err(vec![TypeErr::new(
                            &arg.pos,
                            &format!("{} cannot have default argument", SELF)
                        )]);
                    }

                    res.1 = res.1.insert_new(*mutable, SELF);
                    res.0 = res.0.add(&Expression { ast: *var.clone() }, self_type)
                } else {
                    res.1 = identifier_from_var(var, *mutable, &res.1)?;
                }

                match (ty, default) {
                    (Some(ty), Some(default)) =>
                        res = constrain_ty(default, ty, env, ctx, &constr)?,
                    (Some(ty), None) => res = generate(ty, &res.1, ctx, &res.0)?,
                    (None, Some(expr)) => res = generate(expr, &res.1, ctx, &res.0)?,
                    _ => {}
                }
            }
            _ => return Err(vec![TypeErr::new(&arg.pos, "Expected function argument")])
        }
    }

    Ok(res)
}

pub fn identifier_from_var(
    var: &AST,
    mutable: bool,
    env: &Environment
) -> Constrained<Environment> {
    let identifier = Identifier::try_from(var.deref())?.as_mutable(mutable);
    let mut env = env.clone();
    for (f_mut, f_name) in &identifier.fields() {
        // TODO add Expect binding to environment
        env = env.insert_new(*f_mut, f_name);
    }
    Ok(env)
}
