use std::convert::TryFrom;
use std::ops::Deref;

use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::constraint::builder::ConstrBuilder;
use crate::type_checker::constraints::constraint::expected::Expect::*;
use crate::type_checker::constraints::constraint::expected::Expected;
use crate::type_checker::constraints::generate::generate;
use crate::type_checker::constraints::generate::resources::constrain_raises;
use crate::type_checker::constraints::generate::ty::constrain_ty;
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::function_arg::concrete::SELF;
use crate::type_checker::context::Context;
use crate::type_checker::environment::name::{match_type, Identifier};
use crate::type_checker::environment::Environment;
use crate::type_checker::type_name::TypeName;
use crate::type_checker::type_result::TypeErr;

pub fn gen_def(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder
) -> Constrained {
    match &ast.node {
        Node::FunDef { fun_args, ret_ty, body, raises, .. } => {
            constr.new_set(true);

            let (mut constr, env) = constrain_args(fun_args, env, ctx, constr)?;
            let (mut constr, env) = match (ret_ty, body) {
                (Some(ret_ty), Some(body)) => {
                    let type_name = TypeName::try_from(ret_ty)?;
                    let ret_ty_exp = Expected::new(&ret_ty.pos, &Type { type_name });

                    let env = env.new_state(&env.state.ret_ty(&ret_ty_exp));
                    let (mut constr, env) = constrain_raises(body, raises, &env, ctx, &constr)?;
                    let (constr, _) = constrain_ty(body, ret_ty, &env, ctx, &mut constr)?;
                    (constr, env)
                }
                (None, Some(body)) => (generate(body, &env, ctx, &mut constr)?.0, env.clone()),
                _ => (constr, env)
            };

            constr.exit_set(&ast.pos)?;
            Ok((constr, env))
        }
        Node::FunArg { .. } =>
            Err(vec![TypeErr::new(&ast.pos, "Function argument cannot be top level")]),

        Node::VariableDef { mutable, var, ty, expression: Some(expr), .. } => {
            let (mut constr, env) = identifier_from_var(var, ty, *mutable, constr, env)?;
            let var_expect = Expected::from(var);
            let expr_expect = Expected::from(expr);
            constr.add(&var_expect, &expr_expect);

            match ty {
                Some(ty) => constrain_ty(expr, ty, &env, ctx, &mut constr),
                None => generate(expr, &env, ctx, &mut constr)
            }
        }
        Node::VariableDef { mutable, var, ty, .. } =>
            identifier_from_var(var, ty, *mutable, constr, env),

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected definition")])
    }
}

pub fn constrain_args(
    args: &[AST],
    env: &Environment,
    ctx: &Context,
    constr: &ConstrBuilder
) -> Constrained {
    let mut res = (constr.clone(), env.clone());
    for arg in args {
        match &arg.node {
            Node::FunArg { mutable, var, ty, default, .. } => {
                if var.node == Node::_Self {
                    let self_type = &env.state.in_class_new.clone().ok_or_else(|| {
                        TypeErr::new(&var.pos, &format!("{} cannot be outside class", SELF))
                    })?;
                    if default.is_some() {
                        let msg = format!("{} cannot have default argument", SELF);
                        return Err(vec![TypeErr::new(&arg.pos, &msg)]);
                    }

                    res.1 = res.1.insert_new(*mutable, SELF, self_type);
                    let left = Expected::from(var);
                    res.0.add(&left, &Expected::new(&var.pos, self_type));
                } else {
                    res = identifier_from_var(var, ty, *mutable, &mut res.0, &res.1)?;
                }

                res = match (ty, default) {
                    (Some(ty), Some(default)) => constrain_ty(default, ty, env, ctx, &mut res.0)?,
                    (None, Some(default)) => generate(default, &res.1, ctx, &mut res.0)?,
                    _ => res
                }
            }
            _ => return Err(vec![TypeErr::new(&arg.pos, "Expected function argument")])
        }
    }

    Ok(res)
}

pub fn identifier_from_var(
    var: &AST,
    ty: &Option<Box<AST>>,
    mutable: bool,
    constr: &mut ConstrBuilder,
    env: &Environment
) -> Constrained {
    let mut constr = constr.clone();
    let mut env = env.clone();

    if let Some(ty) = ty {
        let type_name = TypeName::try_from(ty.deref())?;
        let left = Expected::from(var);
        constr.add(&left, &Expected::new(&ty.pos, &Type { type_name: type_name.clone() }));

        let identifier = Identifier::try_from(var.deref())?.as_mutable(mutable);
        for (f_name, (f_mut, type_name)) in match_type(&identifier, &type_name, &var.pos)? {
            env = env.insert_new(mutable && f_mut, &f_name, &Type { type_name });
        }
    } else {
        let identifier = Identifier::try_from(var.deref())?.as_mutable(mutable);
        for (f_mut, f_name) in identifier.fields() {
            env = env.insert_new(mutable && f_mut, &f_name, &ExpressionAny);
        }
    };

    Ok((constr, env))
}
