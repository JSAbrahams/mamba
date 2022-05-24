use std::convert::TryFrom;
use std::ops::Deref;

use permutate::Permutator;

use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::expected::{Expect, Expected};
use crate::check::constrain::constraint::expected::Expect::*;
use crate::check::constrain::generate::{Constrained, generate};
use crate::check::constrain::generate::env::Environment;
use crate::check::context::{arg, clss, Context, function, LookupClass};
use crate::check::context::arg::SELF;
use crate::check::context::clss::HasParent;
use crate::check::context::field::Field;
use crate::check::ident::{IdentiCall, Identifier};
use crate::check::name::{match_name, Union};
use crate::check::name::{IsNullable, Name};
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
            constr.new_set(true);

            let non_nullable_class_vars: Vec<String> = match &id.node {
                Node::Init => {
                    if let Some(class) = constr.current_class() {
                        let class = ctx.class(&class, &id.pos)?;
                        let fields: Vec<&Field> =
                            class.fields.iter().filter(|f| !f.ty.is_nullable()).collect();
                        fields.iter().map(|f| f.name.clone()).collect()
                    } else {
                        let msg = format!("Cannot have {} function outside class", function::INIT);
                        return Err(vec![TypeErr::new(&id.pos, &msg)]);
                    }
                }
                _ => vec![],
            };

            let (mut constr, inner_env) = constrain_args(fun_args, env, ctx, constr)?;
            let inner_env = inner_env.with_unassigned(non_nullable_class_vars);

            let (mut constr, inner_env) = if let Some(body) = body {
                let r_tys: Vec<_> =
                    raises.iter().map(|r| (r.pos.clone(), Name::try_from(r))).collect();
                let mut r_res = Name::empty();

                let exception_name = Name::from(clss::EXCEPTION);
                for (pos, raise) in r_tys {
                    let raise = raise?;
                    if !ctx.class(&raise, &pos)?.has_parent(&exception_name, ctx, &pos)? {
                        let msg = format!("`{}` is not an `{}`", raise, exception_name);
                        return Err(vec![TypeErr::new(&pos, &msg)]);
                    }
                    r_res = r_res.union(&raise)
                }

                let inner_env = inner_env.insert_raises(&r_res, &ast.pos);
                if let Some(ret_ty) = ret_ty {
                    let name = Name::try_from(ret_ty)?;
                    let ret_ty_exp = Expected::new(&ret_ty.pos, &Type { name });
                    let inner_env = inner_env.return_type(&ret_ty_exp);
                    generate(body, &inner_env, ctx, &mut constr)?
                } else {
                    generate(body, &inner_env, ctx, &mut constr)?
                }
            } else {
                (constr, inner_env)
            };

            let unassigned: Vec<String> = inner_env
                .unassigned
                .iter()
                .map(|v| format!("Non nullable class variable '{}' should be assigned to", v))
                .collect();
            if !unassigned.is_empty() {
                return Err(unassigned.iter().map(|msg| TypeErr::new(&id.pos, &msg)).collect());
            }

            constr.exit_set(&ast.pos)?;
            Ok((constr, env.clone()))
        }
        Node::FunArg { .. } => {
            Err(vec![TypeErr::new(&ast.pos, "Function argument cannot be top level")])
        }

        Node::VariableDef { mutable, var, ty, expr: expression, .. } => {
            identifier_from_var(var, ty, expression, *mutable, ctx, constr, env)
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected definition")]),
    }
}

pub fn constrain_args(
    args: &[AST],
    env: &Environment,
    ctx: &Context,
    constr: &ConstrBuilder,
) -> Constrained {
    let mut res = (constr.clone(), env.clone());
    for arg in args {
        match &arg.node {
            Node::FunArg { mutable, var, ty, default, .. } => {
                if var.node == Node::_Self {
                    let self_type = &env.class_type.clone().ok_or_else(|| {
                        TypeErr::new(&var.pos, &format!("{} cannot be outside class", SELF))
                    })?;
                    if default.is_some() {
                        let msg = format!("{} cannot have default argument", SELF);
                        return Err(vec![TypeErr::new(&arg.pos, &msg)]);
                    }

                    let self_exp = Expected::new(&var.pos, self_type);
                    res.1 = res.1.insert_var(*mutable, SELF, &self_exp);
                    let left = Expected::try_from((var, &env.var_mappings))?;
                    res.0.add("arguments", &left, &Expected::new(&var.pos, self_type));
                } else {
                    res = identifier_from_var(var, ty, default, *mutable, ctx, &mut res.0, &res.1)?;
                }
            }
            _ => return Err(vec![TypeErr::new(&arg.pos, "Expected function argument")]),
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
    env: &Environment,
) -> Constrained {
    let (mut constr, mut env) = if let Some(expr) = expression {
        generate(expr, env, ctx, constr)?
    } else {
        (constr.clone(), env.clone())
    };

    let identifier = Identifier::try_from(var.deref())?.as_mutable(mutable);
    if let Some(ty) = ty {
        let name = Name::try_from(ty.deref())?;
        for (f_name, (f_mut, name)) in match_name(&identifier, &name, &var.pos)? {
            let ty = Expected::new(&var.pos, &Type { name: name.clone() });
            env = env.insert_var(mutable && f_mut, &f_name, &ty);
        }
    } else {
        let any = Expected::new(&var.pos, &ExpressionAny);
        for (f_mut, f_name) in identifier.fields(&var.pos)? {
            env = env.insert_var(mutable && f_mut, &f_name, &any);
        }
    };

    if identifier.is_tuple() {
        let tup_exps = identifier_to_tuple(&var.pos, &identifier, &env)?;
        match (ty, expression) {
            (Some(ty), _) => {
                let ty_exp = Expected::try_from((ty, &env.var_mappings))?;
                for tup_exp in tup_exps {
                    constr.add("type and tuple", &ty_exp, &tup_exp);
                }
            }
            (_, Some(expr)) => {
                let expr_expt = Expected::try_from((expr, &env.var_mappings))?;
                if tup_exps.len() > 1 {
                    for tup_exp in &tup_exps {
                        println!("{:?}", tup_exp)
                    }
                    panic!()
                }
                for tup_exp in tup_exps {
                    constr.add("tuple and expression", &expr_expt, &tup_exp);
                }
            }
            _ => {}
        }
    }

    let env = identifier
        .all_calls()
        .iter()
        .map(|call| call.without_obj(arg::SELF, &var.pos))
        .filter(Result::is_ok)
        .map(Result::unwrap)
        .map(|identi_call| match identi_call {
            IdentiCall::Iden(var) => Some(var),
            _ => None,
        })
        .filter(Option::is_some)
        .map(Option::unwrap)
        .fold(env, |env, self_var| env.assigned_to(&self_var));

    let var_expect = Expected::try_from((var, &env.var_mappings))?;
    match (ty, expression) {
        (Some(ty), Some(expr)) => {
            let ty_exp = Type { name: Name::try_from(ty.deref())? };
            let parent = Expected::new(&ty.pos, &ty_exp);
            constr.add("variable, type, and expression", &parent, &var_expect);
            let expr_expect = Expected::try_from((expr, &env.var_mappings))?;
            constr.add("variable, type, and expression", &var_expect, &expr_expect);
        }
        (Some(ty), None) => {
            let ty_exp = Type { name: Name::try_from(ty.deref())? };
            let parent = Expected::new(&ty.pos, &ty_exp);
            constr.add("variable with type", &parent, &var_expect);
        }
        (None, Some(expr)) => {
            let parent = Expected::try_from((expr, &env.var_mappings))?;
            constr.add("variable and expression", &parent, &var_expect);
        }
        (None, None) => {
            let child = Expected::new(&var.pos, &ExpressionAny);
            constr.add("variable", &var_expect, &child);
        }
    };

    Ok((constr, env.clone()))
}

// Returns every possible tuple. Elements of a tuple are not to be confused with
// the union of types derived from the current environment.
fn identifier_to_tuple(
    pos: &Position,
    iden: &Identifier,
    env: &Environment,
) -> TypeResult<Vec<Expected>> {
    match &iden {
        Identifier::Single(_, var) => {
            if let Some(expected) = env.get_var(&var.object(pos)?) {
                Ok(expected.iter().map(|(_, exp)| exp.clone()).collect())
            } else {
                let msg = format!("'{}' is undefined in this scope", iden);
                Err(vec![TypeErr::new(pos, &msg)])
            }
        }
        Identifier::Multi(idens) => {
            // Every item in the tuple is a union of expected
            let tuple_unions: Vec<Vec<Expected>> =
                idens.iter().map(|i| identifier_to_tuple(pos, i, env)).collect::<Result<_, _>>()?;

            // .. So we create permutation of every possible tuple combination
            let tuple_unions: Vec<Vec<&Expected>> =
                tuple_unions.iter().map(|list| list.iter().map(AsRef::as_ref).collect()).collect();
            let tuple_unions: Vec<&[&Expected]> = tuple_unions.iter().map(AsRef::as_ref).collect();
            let permutations = Permutator::new(&tuple_unions[..]);

            Ok(permutations
                .into_iter()
                .map(|elements| {
                    let elements = elements.into_iter().cloned().collect();
                    Expected::new(pos, &Expect::Tuple { elements })
                })
                .collect())
        }
    }
}
