use std::collections::HashSet;
use std::convert::TryFrom;
use std::ops::Deref;

use itertools::enumerate;

use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::expected::Expect::*;
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::generate::env::Environment;
use crate::check::constrain::generate::{generate, Constrained};
use crate::check::context::arg::SELF;
use crate::check::context::clss::{Class, HasParent};
use crate::check::context::field::Field;
use crate::check::context::function::python::INIT;
use crate::check::context::{clss, Context, LookupClass};
use crate::check::ident::Identifier;
use crate::check::name::true_name::TrueName;
use crate::check::name::{match_name, Name, Nullable, TupleCallable};
use crate::check::result::{TypeErr, TypeResult};
use crate::common::position::Position;
use crate::parse::ast::Node::Id;
use crate::parse::ast::{Node, AST};

pub fn gen_def(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    match &ast.node {
        Node::FunDef {
            args: fun_args,
            ret: ret_ty,
            body,
            raises,
            id,
            ..
        } => {
            let (class, non_nullable_class_vars) = match &id.node {
                Id { lit } if *lit == INIT => {
                    if let Some(class) = &env.class {
                        let class = ctx.class(class, id.pos)?;
                        let parents: Vec<Class> = class
                            .parents
                            .iter()
                            .map(|p| ctx.class(p, id.pos))
                            .collect::<TypeResult<_>>()?;

                        let fields: Vec<&Field> = class
                            .fields
                            .iter()
                            .filter(|f| !parents.iter().any(|p| p.fields.contains(f)))
                            .filter(|f| !f.ty.is_nullable() && !f.assigned_to)
                            .collect();
                        (
                            Some(class.clone()),
                            fields.iter().map(|f| f.name.clone()).collect(),
                        )
                    } else {
                        let msg = format!("Cannot have {INIT} function outside class");
                        return Err(vec![TypeErr::new(id.pos, &msg)]);
                    }
                }
                _ => (None, HashSet::new()),
            };

            let body_env = constrain_args(fun_args, env, ctx, constr)?
                .with_unassigned(non_nullable_class_vars)
                .in_fun(true);

            let (raises, errs): (Vec<(Position, _)>, Vec<_>) = raises
                .iter()
                .map(|r| (r.pos, TrueName::try_from(r)))
                .partition(|(_, res)| res.is_ok());
            if !errs.is_empty() {
                let errs = errs.into_iter().flat_map(|(_, e)| e.unwrap_err());
                return Err(errs.collect());
            }

            let exception_name = Name::from(clss::EXCEPTION);
            for (pos, raise) in &raises {
                let raise = raise.clone()?;
                if !ctx
                    .class(&raise, *pos)?
                    .has_parent(&exception_name, ctx, *pos)?
                {
                    let msg = format!("`{raise}` is not an `{exception_name}`");
                    return Err(vec![TypeErr::new(*pos, &msg)]);
                }
            }

            let raises = raises.into_iter().map(|(_, r)| r.unwrap()).collect();
            let body_env = body_env.raises_caught(&raises);

            let body_env = if let Some(body) = body {
                if let Some(ret_ty) = ret_ty {
                    let name = Name::try_from(ret_ty)?;
                    let ret_ty_raises_exp = Expected::new(body.pos, &Type { name: name.clone() });
                    constr.add(
                        "fun body type",
                        &ret_ty_raises_exp,
                        &Expected::from(body),
                        env,
                    );

                    let ret_ty_exp = Expected::new(ret_ty.pos, &Type { name });
                    let body_env = body_env.return_type(&ret_ty_exp).is_expr(true);
                    generate(body, &body_env, ctx, constr)?
                } else {
                    generate(body, &body_env, ctx, constr)?
                }
            } else {
                body_env
            };

            if let Some(class) = class {
                let unassigned: Vec<String> = body_env
                    .unassigned
                    .iter()
                    .map(|v| format!("Non nullable attribute '{v}' of {class} not assigned to in constructor"))
                    .collect();
                if !unassigned.is_empty() {
                    return Err(unassigned
                        .iter()
                        .map(|msg| TypeErr::new(id.pos, msg))
                        .collect());
                }
            }

            Ok(env.clone())
        }
        Node::FunArg { .. } => Err(vec![TypeErr::new(
            ast.pos,
            "Function argument cannot be top level",
        )]),

        Node::VariableDef {
            mutable,
            var,
            ty,
            expr: expression,
            ..
        } => {
            if let Some(ty) = ty {
                let name = Name::try_from(ty)?;
                id_from_var(var, &Some(name), expression, *mutable, ctx, constr, env)
            } else {
                id_from_var(var, &None, expression, *mutable, ctx, constr, env)
            }
        }

        _ => Err(vec![TypeErr::new(ast.pos, "Expected definition")]),
    }
}

pub fn constrain_args(
    args: &[AST],
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    let exp_expression = env.is_expr;
    let mut env_with_args = env.is_expr(true);

    for arg in args {
        match &arg.node {
            Node::FunArg {
                mutable,
                var,
                ty,
                default,
                ..
            } => {
                if var.node == Node::new_self() {
                    let class_name = &env.class.clone().ok_or_else(|| {
                        TypeErr::new(var.pos, &format!("{SELF} cannot be outside class"))
                    })?;
                    if default.is_some() {
                        let msg = format!("{SELF} cannot have default argument");
                        return Err(vec![TypeErr::new(arg.pos, &msg)]);
                    }

                    let name = Some(if let Some(ty) = ty {
                        Name::try_from(ty)?
                    } else {
                        Name::from(class_name)
                    });
                    env_with_args =
                        id_from_var(var, &name, default, *mutable, ctx, constr, &env_with_args)?
                } else {
                    let ty = if let Some(ty) = ty {
                        Some(Name::try_from(ty)?)
                    } else {
                        None
                    };
                    env_with_args =
                        id_from_var(var, &ty, default, *mutable, ctx, constr, &env_with_args)?;
                }
            }
            _ => {
                let msg = format!("Expected function argument, was {}", arg.node);
                return Err(vec![TypeErr::new(arg.pos, &msg)]);
            }
        }
    }

    Ok(env_with_args.is_expr(exp_expression))
}

pub fn id_from_var(
    var: &AST,
    ty: &Option<Name>,
    expr: &Option<Box<AST>>,
    mutable: bool,
    ctx: &Context,
    constr: &mut ConstrBuilder,
    env: &Environment,
) -> Constrained {
    if let Some(expr) = expr {
        generate(expr, &env.is_expr(true), ctx, constr)?;
    }

    let mut env = env.clone();
    let identifier = Identifier::try_from(var.deref())?.as_mutable(mutable);
    match (ty, expr) {
        (Some(ty), Some(expr)) => {
            let mut names = vec![];
            for (f_name, (f_mut, name)) in match_name(&identifier, ty, var.pos)? {
                names.push(name.clone());

                constr.insert_var(&f_name);
                let ty = Expected::new(var.pos, &Type { name: name.clone() });
                env = env.insert_var(mutable && f_mut, &f_name, &ty, &constr.var_mapping);
            }

            let ty_exp = Expected::new(var.pos, &Type { name: ty.clone() });
            constr.add(
                "variable with type and...",
                &ty_exp,
                &Expected::from(var),
                &env,
            );
            constr.add(
                "variable with expression",
                &ty_exp,
                &Expected::from(expr),
                &env,
            );
        }
        (Some(ty), None) => {
            for (f_name, (f_mut, name)) in match_name(&identifier, ty, var.pos)? {
                constr.insert_var(&f_name);
                let ty = Expected::new(var.pos, &Type { name: name.clone() });
                env = env.insert_var(mutable && f_mut, &f_name, &ty, &constr.var_mapping);
            }

            let ty_exp = Expected::new(var.pos, &Type { name: ty.clone() });
            constr.add(
                "variable with only type",
                &ty_exp,
                &Expected::from(var),
                &env,
            );
        }
        (None, Some(expr)) => {
            let mut temp_names = vec![];
            let fields = identifier.fields(var.pos)?;
            for (f_mut, name) in &fields {
                let temp_name = constr.temp_name();
                temp_names.push(temp_name.clone());

                constr.insert_var(name);
                let ty = Expected::new(
                    var.pos,
                    &Type {
                        name: temp_name.clone(),
                    },
                );
                env = env.insert_var(mutable && *f_mut, name, &ty, &constr.var_mapping);

                let var = AST::new(var.pos, Id { lit: name.clone() });
                constr.add(
                    "variable with only expression",
                    &Expected::from(&var),
                    &ty,
                    &env,
                );
            }

            let exp_expr = if temp_names.len() > 1 {
                // if tuple literal, deconstruct elements in generate stage
                if let Node::Tuple { elements } = &expr.node {
                    if elements.len() == temp_names.len() {
                        for (i, (expr, ty)) in enumerate(elements.iter().zip(&temp_names)) {
                            let expr_exp = Expected::from(expr);
                            let expr_ty = Expected::new(expr.pos, &Type { name: ty.clone() });

                            let msg = format!("tuple literal element {i}");
                            constr.add(&msg, &expr_ty, &expr_exp, &env);
                        }
                    } else {
                        let msg = format!(
                            "Expected tuple of {} elements, was {}",
                            temp_names.len(),
                            elements.len()
                        );
                        return Err(vec![TypeErr::new(expr.pos, &msg)]);
                    }
                }

                Expected::new(
                    expr.pos,
                    &Type {
                        name: Name::tuple(&temp_names),
                    },
                )
            } else if let Some(first) = temp_names.first() {
                Expected::new(
                    expr.pos,
                    &Type {
                        name: first.clone(),
                    },
                )
            } else {
                panic!("cannot have empty identifier")
            };

            constr.add(
                "variable with only expression",
                &Expected::from(var),
                &Expected::from(expr),
                &env,
            );
            constr.add(
                "variable with only expression",
                &exp_expr,
                &Expected::from(expr),
                &env,
            );
        }
        (None, None) => {
            let any = Expected::any(var.pos);
            for (f_mut, f_name) in identifier.fields(var.pos)? {
                constr.insert_var(&f_name);
                env = env.insert_var(mutable && f_mut, &f_name, &any, &constr.var_mapping);
            }
        }
    }

    Ok(env)
}
