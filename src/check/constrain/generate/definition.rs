use std::collections::HashSet;
use std::convert::TryFrom;
use std::ops::Deref;

use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::expected::Expect::*;
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::generate::{Constrained, generate};
use crate::check::constrain::generate::env::Environment;
use crate::check::context::{clss, Context, function, LookupClass};
use crate::check::context::arg::SELF;
use crate::check::context::clss::HasParent;
use crate::check::context::field::Field;
use crate::check::ident::Identifier;
use crate::check::name::{match_name, Name, Nullable};
use crate::check::name::true_name::TrueName;
use crate::check::result::{TypeErr, TypeResult};
use crate::common::position::Position;
use crate::parse::ast::{AST, Node};

pub fn gen_def(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    match &ast.node {
        Node::FunDef { args: fun_args, ret: ret_ty, body, raises, id, .. } => {
            let non_nullable_class_vars: HashSet<String> = match &id.node {
                Node::Id { lit } if *lit == function::INIT => {
                    if let Some(class) = &env.class {
                        let class = ctx.class(class, id.pos)?;
                        let fields: Vec<&Field> = class
                            .fields
                            .iter()
                            .filter(|f| !f.ty.is_nullable() && !f.assigned_to)
                            .collect();
                        fields.iter().map(|f| f.name.clone()).collect()
                    } else {
                        let msg = format!("Cannot have {} function outside class", function::INIT);
                        return Err(vec![TypeErr::new(id.pos, &msg)]);
                    }
                }
                _ => HashSet::new(),
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
                if !ctx.class(&raise, *pos)?.has_parent(&exception_name, ctx, *pos)? {
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
                    constr.add("fun body type", &ret_ty_raises_exp, &Expected::from(body), env);

                    let ret_ty_exp = Expected::new(ret_ty.pos, &Type { name });
                    let body_env = body_env.return_type(&ret_ty_exp).is_expr(true);
                    generate(body, &body_env, ctx, constr)?
                } else {
                    generate(body, &body_env, ctx, constr)?
                }
            } else {
                body_env
            };

            let unassigned: Vec<String> = body_env
                .unassigned
                .iter()
                .map(|v| format!("Non nullable class variable '{v}' not assigned to in constructor"))
                .collect();
            if !unassigned.is_empty() {
                return Err(unassigned.iter().map(|msg| TypeErr::new(id.pos, msg)).collect());
            }

            Ok(env.clone())
        }
        Node::FunArg { .. } => {
            Err(vec![TypeErr::new(ast.pos, "Function argument cannot be top level")])
        }

        Node::VariableDef { mutable, var, ty, expr: expression, .. } => if let Some(ty) = ty {
            let name = Name::try_from(ty)?;
            id_from_var(var, &Some(name), expression, *mutable, ctx, constr, env)
        } else {
            id_from_var(var, &None, expression, *mutable, ctx, constr, env)
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
            Node::FunArg { mutable, var, ty, default, .. } => {
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
                    env_with_args = id_from_var(var, &name, default, *mutable, ctx, constr, &env_with_args)?
                } else {
                    let ty = if let Some(ty) = ty { Some(Name::try_from(ty)?) } else { None };
                    env_with_args = id_from_var(var, &ty, default, *mutable, ctx, constr, &env_with_args)?;
                }
            }
            _ => return Err(vec![TypeErr::new(arg.pos, "Expected function argument")]),
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
    if let Some(ty) = ty {
        for (f_name, (f_mut, name)) in match_name(&identifier, ty, var.pos)? {
            let ty = Expected::new(var.pos, &Type { name: name.clone() });
            constr.insert_var(&f_name);
            env = env.insert_var(mutable && f_mut, &f_name, &ty, &constr.var_mapping);
        }
    } else {
        let any = Expected::any(var.pos);
        for (f_mut, f_name) in identifier.fields(var.pos)? {
            constr.insert_var(&f_name);
            env = env.insert_var(mutable && f_mut, &f_name, &any, &constr.var_mapping);
        }
    };

    if identifier.is_tuple() {
        let tup_exps = identifier_to_tuple(var.pos, &identifier, &env, constr)?;
        if let Some(ty) = ty {
            let ty_exp = Expected::new(var.pos, &Type { name: ty.clone() });
            for tup_exp in &tup_exps {
                constr.add("type and tuple", &ty_exp, tup_exp, &env);
            }
        }
        if let Some(expr) = expr {
            for tup_exp in &tup_exps {
                constr.add("tuple and expression", &Expected::from(expr), tup_exp, &env);
            }
        }
    }

    if let Some(ty) = ty {
        let ty_exp = Expected::new(var.pos, &Type { name: ty.clone() });
        constr.add("variable and type", &ty_exp, &Expected::from(var), &env);
    }
    if let Some(expr) = expr {
        let msg = format!("variable and expression: `{}`", expr.node);
        constr.add(&msg, &Expected::from(var), &Expected::from(expr), &env);
    }

    Ok(env)
}

// Returns every possible tuple. Elements of a tuple are not to be confused with
// the union of types derived from the current environment.
fn identifier_to_tuple(
    pos: Position,
    iden: &Identifier,
    env: &Environment,
    constr: &mut ConstrBuilder,
) -> TypeResult<Vec<Expected>> {
    match &iden {
        Identifier::Single(_, var) => {
            if let Some(expected) = env.get_var(&var.object(pos)?, &constr.var_mapping) {
                Ok(expected.iter().map(|(_, exp)| exp.clone()).collect())
            } else {
                let msg = format!("'{iden}' is undefined in this scope");
                Err(vec![TypeErr::new(pos, &msg)])
            }
        }
        Identifier::Multi(_idens) => {
            unimplemented!()
        }
    }
}
