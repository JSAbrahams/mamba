use std::convert::TryFrom;
use std::ops::Deref;

use crate::parser::ast::{Node, AST};
use crate::type_checker::checker_result::TypeErr;
use crate::type_checker::constraints::constraint::builder::ConstrBuilder;
use crate::type_checker::constraints::constraint::expected::Expect::*;
use crate::type_checker::constraints::constraint::expected::Expected;
use crate::type_checker::constraints::generate::generate;
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::function_arg::concrete::SELF;
use crate::type_checker::context::{ty, Context};
use crate::type_checker::environment::name::{match_type, Identifier};
use crate::type_checker::environment::Environment;
use crate::type_checker::ty_name::TypeName;
use std::collections::HashSet;

pub fn gen_def(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder
) -> Constrained {
    match &ast.node {
        Node::FunDef { fun_args, ret_ty, body, raises, .. } => {
            constr.new_set(true);

            let (mut constr, inner_env) = constrain_args(fun_args, env, ctx, constr)?;
            let mut constr = if let Some(body) = body {
                let r_tys: Vec<_> =
                    raises.into_iter().map(|r| (r.pos.clone(), TypeName::try_from(r))).collect();
                let mut r_res: HashSet<TypeName> = HashSet::new();
                let exception_ty = TypeName::from(ty::concrete::EXCEPTION);
                for (pos, raise) in r_tys {
                    let raise = raise?;
                    if !ctx.lookup(&raise, &pos)?.has_parent(&exception_ty, ctx, &pos)? {
                        let msg = format!("{} is not an {}", raise, ty::concrete::EXCEPTION);
                        return Err(vec![TypeErr::new(&pos, &msg)]);
                    }
                    r_res.insert(raise);
                }

                let inner_env = inner_env.insert_raises(&r_res, &ast.pos);
                if let Some(ret_ty) = ret_ty {
                    let type_name = TypeName::try_from(ret_ty)?;
                    let ret_ty_exp = Expected::new(&ret_ty.pos, &Type { type_name });
                    let inner_env = inner_env.return_type(&ret_ty_exp);
                    generate(body, &inner_env, ctx, &mut constr)?.0
                } else {
                    generate(body, &inner_env, ctx, &mut constr)?.0
                }
            } else {
                constr
            };

            constr.exit_set(&ast.pos)?;
            Ok((constr, env.clone()))
        }
        Node::FunArg { .. } =>
            Err(vec![TypeErr::new(&ast.pos, "Function argument cannot be top level")]),

        Node::VariableDef { mutable, var, ty, expression, .. } =>
            identifier_from_var(var, ty, expression, *mutable, ctx, constr, env),

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
            Node::FunArg { mutable, var, ty, default, .. } =>
                if var.node == Node::_Self {
                    let self_type = &env.class_type.clone().ok_or_else(|| {
                        TypeErr::new(&var.pos, &format!("{} cannot be outside class", SELF))
                    })?;
                    if default.is_some() {
                        let msg = format!("{} cannot have default argument", SELF);
                        return Err(vec![TypeErr::new(&arg.pos, &msg)]);
                    }

                    let self_exp = Expected::new(&var.pos, &self_type);
                    res.1 = res.1.insert_var(*mutable, SELF, &self_exp);
                    let left = Expected::from(var);
                    res.0.add(&left, &Expected::new(&var.pos, self_type));
                } else {
                    res = identifier_from_var(var, ty, default, *mutable, ctx, &mut res.0, &res.1)?;
                },
            _ => return Err(vec![TypeErr::new(&arg.pos, "Expected function argument")])
        }
    }

    Ok(res)
}

pub fn identifier_from_var(
    var: &AST,
    ty: &Option<Box<AST>>,
    expression: &Option<Box<AST>>,
    mutable: bool,
    ctx: &Context,
    constr: &mut ConstrBuilder,
    env: &Environment
) -> Constrained {
    let mut constr = constr.clone();
    let mut env = env.clone();
    let mut names = vec![];

    if let Some(ty) = ty {
        let type_name = TypeName::try_from(ty.deref())?;
        let identifier = Identifier::try_from(var.deref())?.as_mutable(mutable);
        for (f_name, (f_mut, type_name)) in match_type(&identifier, &type_name, &var.pos)? {
            let ty = Expected::new(&var.pos, &Type { type_name: type_name.clone() });
            env = env.insert_var(mutable && f_mut, &f_name, &ty);
            names.push(f_name);
        }
    } else {
        let identifier = Identifier::try_from(var.deref())?.as_mutable(mutable);
        let any = Expected::new(&var.pos, &ExpressionAny);
        for (f_mut, f_name) in identifier.fields() {
            env = env.insert_var(mutable && f_mut, &f_name, &any);
            names.push(f_name);
        }
    };

    let var_expect = Expected::from(var);
    match (ty, expression) {
        (Some(ty), Some(expr)) => {
            let type_name = TypeName::try_from(ty.deref())?;
            constr.add_with_identifier(
                &var_expect,
                &Expected::new(&ty.pos, &Type { type_name }),
                &names
            );
            let expr_expect = Expected::from(expr);
            constr.add(&var_expect, &expr_expect);
            generate(expr, &env, ctx, &mut constr)
        }
        (Some(ty), None) => {
            let type_name = TypeName::try_from(ty.deref())?;
            constr.add_with_identifier(
                &var_expect,
                &Expected::new(&ty.pos, &Type { type_name }),
                &names
            );
            Ok((constr, env))
        }
        (None, Some(expr)) => {
            constr.add_with_identifier(&var_expect, &Expected::from(expr), &names);
            generate(expr, &env, ctx, &mut constr)
        }
        (None, None) => {
            constr.add_with_identifier(
                &var_expect,
                &Expected::new(&var.pos, &ExpressionAny),
                &names
            );
            Ok((constr, env))
        }
    }
}
