use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::cons::Constraints;
use crate::type_checker::constraints::cons::Expect::{Expression, Type};
use crate::type_checker::constraints::generate::generate;
use crate::type_checker::constraints::generate::ty::constrain_ty;
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::Context;
use crate::type_checker::environment::name::Identifier;
use crate::type_checker::environment::Environment;
use crate::type_checker::type_name::TypeName;
use crate::type_checker::type_result::TypeErr;
use std::convert::TryFrom;
use std::ops::Deref;

pub fn gen_def(ast: &AST, env: &Environment, ctx: &Context, constr: &Constraints) -> Constrained {
    match &ast.node {
        Node::FunDef { fun_args, ret_ty, body, .. } => {
            let (constr, env) = constrain_args(fun_args, env, ctx, constr)?;
            match (ret_ty, body) {
                (Some(ret_ty), Some(body)) => constrain_ty(body, ret_ty, &env, ctx, &constr),
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
                (Some(ty), Some(expr)) => constrain_ty(expr, ty, &env, ctx, constr),
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
    let mut constr_env = (constr.clone(), env.clone());
    for arg in args {
        match &arg.node {
            Node::FunArg { mutable, var, ty, default, .. } => {
                let identifier = Identifier::try_from(var.deref())?.as_mutable(*mutable);
                for (f_mut, f_name) in &identifier.fields() {
                    constr_env.1 = env.insert_new(*f_mut, f_name);
                }
                match (ty, default) {
                    (Some(ty), Some(default)) =>
                        constr_env = constrain_ty(default, ty, &env, ctx, &constr)?,
                    _ => {}
                }
            }
            _ => return Err(vec![TypeErr::new(&arg.pos, "Expected function argument")])
        }
    }

    Ok(constr_env)
}
