use std::convert::TryFrom;
use std::ops::Deref;

use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::cons::Constraints;
use crate::type_checker::constraints::cons::Expect::Expression;
use crate::type_checker::constraints::generate::resources::constrain_raises;
use crate::type_checker::constraints::generate::ty::constrain_ty;
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::{function_arg, Context};
use crate::type_checker::environment::name::Identifier;
use crate::type_checker::environment::Environment;
use crate::type_checker::type_result::TypeErr;

pub fn gen_def(ast: &AST, env: &Environment, ctx: &Context, constr: &Constraints) -> Constrained {
    match &ast.node {
        Node::FunDef { fun_args, ret_ty, body, raises, .. } => {
            let (constr, env) = constrain_args(fun_args, env, ctx, constr)?;
            match (ret_ty, body) {
                (Some(ret_ty), Some(body)) => {
                    let constr = constrain_raises(body, raises, ctx, &constr)?;
                    Ok((constrain_ty(body, ret_ty, ctx, &constr)?, env))
                }
                _ => Ok((constr.clone(), env.clone()))
            }
        }
        Node::FunArg { .. } =>
            Err(vec![TypeErr::new(&ast.pos, "Function argument cannot be top level")]),

        Node::VariableDef { mutable, var, ty, expression, .. } => {
            let identifier = Identifier::try_from(var.deref())?.as_mutable(*mutable);
            let mut env = env.clone();
            for (f_mut, f_name) in &identifier.fields() {
                // TODO add Expect binding to environment
                env = env.insert_new(*f_mut, f_name);
            }

            match (ty, expression) {
                (Some(ty), Some(expr)) => Ok((constrain_ty(expr, ty, ctx, constr)?, env)),
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
    let (mut constr, mut env) = (constr.clone(), env.clone());
    for arg in args {
        match &arg.node {
            Node::FunArg { mutable, var, ty, default, .. } =>
                if &var.node == &Node::_Self {
                    let self_type = &env.state.in_class_new.clone().ok_or_else(|| {
                        TypeErr::new(
                            &var.pos,
                            &format!("{} cannot be outside class", function_arg::concrete::SELF)
                        )
                    })?;
                    env = env.insert_new(*mutable, function_arg::concrete::SELF);
                    constr = constr.add(&Expression { ast: *var.clone() }, self_type)
                } else {
                    let identifier = Identifier::try_from(var.deref())?.as_mutable(*mutable);
                    for (f_mut, f_name) in &identifier.fields() {
                        env = env.insert_new(*f_mut, f_name);
                    }
                    match (ty, default) {
                        (Some(ty), Some(default)) =>
                            constr = constrain_ty(default, ty, ctx, &constr)?,
                        _ => {}
                    }
                },
            _ => return Err(vec![TypeErr::new(&arg.pos, "Expected function argument")])
        }
    }

    Ok((constr, env))
}
