use std::convert::TryFrom;
use std::ops::Deref;

use permutate::Permutator;

use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::expected::{Expect, Expected};
use crate::check::constrain::constraint::expected::Expect::*;
use crate::check::constrain::generate::{Constrained, generate};
use crate::check::constrain::generate::env::Environment;
use crate::check::context::{clss, Context, function, LookupClass};
use crate::check::context::arg::SELF;
use crate::check::context::clss::HasParent;
use crate::check::context::field::Field;
use crate::check::ident::Identifier;
use crate::check::name::{Any, match_name, Name, Nullable};
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
            constr.new_set(true);

            let non_nullable_class_vars: Vec<String> = match &id.node {
                Node::Id { lit } if *lit == function::INIT => {
                    if let Some(class) = constr.current_class() {
                        let class = ctx.class(&class, id.pos)?;
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
                _ => vec![],
            };

            let (mut constr, inner_env) = constrain_args(fun_args, env, ctx, constr)?;
            let inner_env = inner_env.with_unassigned(non_nullable_class_vars);

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
                    let msg = format!("`{}` is not an `{}`", raise, exception_name);
                    return Err(vec![TypeErr::new(*pos, &msg)]);
                }
            }

            let raises = raises.into_iter().map(|(_, r)| r.unwrap()).collect();
            let inner_env = inner_env.accounted_for_raises(&raises);

            let (mut constr, inner_env) = if let Some(body) = body {
                if let Some(ret_ty) = ret_ty {
                    let name = Name::try_from(ret_ty)?;
                    let ret_ty_raises_exp = Expected::new(body.pos, &Type { name: name.clone() });
                    constr.add("fun body type", &ret_ty_raises_exp, &Expected::try_from((body, &env.var_mappings))?);

                    let ret_ty_exp = Expected::new(ret_ty.pos, &Type { name });
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
                .map(|v| {
                    format!(
                        "Non nullable class variable '{}' should be assigned to in constructor",
                        v
                    )
                })
                .collect();
            if !unassigned.is_empty() {
                return Err(unassigned.iter().map(|msg| TypeErr::new(id.pos, msg)).collect());
            }

            constr.exit_set(ast.pos)?;
            Ok((constr, env.clone()))
        }
        Node::FunArg { .. } => {
            Err(vec![TypeErr::new(ast.pos, "Function argument cannot be top level")])
        }

        Node::VariableDef { mutable, var, ty, expr: expression, .. } => {
            identifier_from_var(var, ty, expression, *mutable, ctx, constr, env)
        }

        _ => Err(vec![TypeErr::new(ast.pos, "Expected definition")]),
    }
}

pub fn constrain_args(
    args: &[AST],
    env: &Environment,
    ctx: &Context,
    constr: &ConstrBuilder,
) -> Constrained {
    let exp_expression = env.exp_expression;
    let mut res = (constr.clone(), env.clone().exp_expression(true));
    let env = env.exp_expression(exp_expression);

    for arg in args {
        match &arg.node {
            Node::FunArg { mutable, var, ty, default, .. } => {
                if var.node == Node::new_self() {
                    let self_type = &env.class_type.clone().ok_or_else(|| {
                        TypeErr::new(var.pos, &format!("{} cannot be outside class", SELF))
                    })?;
                    if default.is_some() {
                        let msg = format!("{} cannot have default argument", SELF);
                        return Err(vec![TypeErr::new(arg.pos, &msg)]);
                    }

                    let self_exp = Expected::new(var.pos, self_type);
                    res.1 = res.1.insert_var(*mutable, SELF, &self_exp);
                    let left = Expected::try_from((var, &env.var_mappings))?;
                    res.0.add("arguments", &left, &Expected::new(var.pos, self_type));
                } else {
                    res = identifier_from_var(var, ty, default, *mutable, ctx, &mut res.0, &res.1)?;
                }
            }
            _ => return Err(vec![TypeErr::new(arg.pos, "Expected function argument")]),
        }
    }

    Ok(res)
}

pub fn identifier_from_var(
    var: &AST,
    ty: &Option<Box<AST>>,
    expr: &Option<Box<AST>>,
    mutable: bool,
    ctx: &Context,
    constr: &mut ConstrBuilder,
    env: &Environment,
) -> Constrained {
    let exp_expression = env.exp_expression;
    let (mut constr, mut env) = if let Some(expr) = expr {
        generate(expr, &env.exp_expression(true), ctx, constr)?
    } else {
        (constr.clone(), env.clone())
    };
    env = env.exp_expression(exp_expression);

    let identifier = Identifier::try_from(var.deref())?.as_mutable(mutable);
    if let Some(ty) = ty {
        let name = Name::try_from(ty.deref())?;
        for (f_name, (f_mut, name)) in match_name(&identifier, &name, var.pos)? {
            let ty = Expected::new(var.pos, &Type { name: name.clone() });
            env = env.insert_var(mutable && f_mut, &f_name, &ty);
        }
    } else {
        let any = Expected::new(var.pos, &Expect::any());
        for (f_mut, f_name) in identifier.fields(var.pos)? {
            env = env.insert_var(mutable && f_mut, &f_name, &any);
        }
    };

    if identifier.is_tuple() {
        let tup_exps = identifier_to_tuple(var.pos, &identifier, &env)?;
        if let Some(ty) = ty {
            let ty_exp = Expected::try_from((ty, &env.var_mappings))?;
            for tup_exp in &tup_exps {
                constr.add("type and tuple", &ty_exp, tup_exp);
            }
        }
        if let Some(expr) = expr {
            let expr_expt = Expected::try_from((expr, &env.var_mappings))?;
            for tup_exp in &tup_exps {
                constr.add("tuple and expression", &expr_expt, tup_exp);
            }
        }
    }

    let var_expect = Expected::try_from((var, &env.var_mappings))?;
    if let Some(ty) = ty {
        let ty_exp = Expected::new(ty.pos, &Type { name: Name::try_from(ty.deref())? });
        constr.add("variable, type, and expression", &ty_exp, &var_expect);
    }
    if let Some(expr) = expr {
        let expr_expect = Expected::try_from((expr, &env.var_mappings))?;
        constr.add("variable, type, and expression", &var_expect, &expr_expect);
    }

    Ok((constr, env))
}

// Returns every possible tuple. Elements of a tuple are not to be confused with
// the union of types derived from the current environment.
fn identifier_to_tuple(
    pos: Position,
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
                    Expected::new(pos, &Tuple { elements })
                })
                .collect())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::check::constrain::constraint::builder::ConstrBuilder;
    use crate::check::constrain::generate::env::Environment;
    use crate::check::constrain::generate::generate;
    use crate::check::context::Context;
    use crate::parse::parse;

    #[test]
    fn if_else_as_expr() {
        let src = "def a := if True then 10 else 20";
        let ast = parse(src).unwrap();
        let (builder, _) = generate(&ast, &Environment::default(), &Context::default(), &mut ConstrBuilder::new()).unwrap();

        // Ignore then and else branches
        let mut constraints = builder.all_constr()[2].clone();

        let mut popped = HashSet::new();
        while let Some(pop) = constraints.pop_constr() {
            popped.insert(pop);
        }
        assert_eq!(popped.len(), 3);
    }
}
