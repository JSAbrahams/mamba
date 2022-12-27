use std::collections::HashSet;
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

            let non_nullable_class_vars: HashSet<String> = match &id.node {
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
                _ => HashSet::new(),
            };

            let body_env = constrain_args(fun_args, env, ctx, constr)?
                .with_unassigned(non_nullable_class_vars);

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
                    constr.add("fun body type", &ret_ty_raises_exp, &Expected::try_from((body, &body_env.var_mappings))?);

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

            constr.exit_set(ast.pos)?;
            Ok(env.clone())
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
    constr: &mut ConstrBuilder,
) -> Constrained {
    let exp_expression = env.is_expr;
    let mut env_with_args = env.is_expr(true);

    for arg in args {
        match &arg.node {
            Node::FunArg { mutable, var, ty, default, .. } => {
                if var.node == Node::new_self() {
                    let self_type = &env_with_args.class_type.clone().ok_or_else(|| {
                        TypeErr::new(var.pos, &format!("{SELF} cannot be outside class"))
                    })?;
                    if default.is_some() {
                        let msg = format!("{SELF} cannot have default argument");
                        return Err(vec![TypeErr::new(arg.pos, &msg)]);
                    }

                    let self_exp = Expected::new(var.pos, self_type);
                    env_with_args = env_with_args.insert_var(*mutable, SELF, &self_exp);
                    let left = Expected::try_from((var, &env_with_args.var_mappings))?;
                    constr.add("arguments", &left, &Expected::new(var.pos, self_type));
                } else {
                    env_with_args = identifier_from_var(var, ty, default, *mutable, ctx, constr, &env_with_args)?;
                }
            }
            _ => return Err(vec![TypeErr::new(arg.pos, "Expected function argument")]),
        }
    }

    Ok(env_with_args.is_expr(exp_expression))
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
    if let Some(expr) = expr {
        generate(expr, &env.is_expr(true), ctx, constr)?;
    }
    let mut env_with_var = env.clone();

    let identifier = Identifier::try_from(var.deref())?.as_mutable(mutable);
    if let Some(ty) = ty {
        let name = Name::try_from(ty.deref())?;
        for (f_name, (f_mut, name)) in match_name(&identifier, &name, var.pos)? {
            let ty = Expected::new(var.pos, &Type { name: name.clone() });
            env_with_var = env_with_var.insert_var(mutable && f_mut, &f_name, &ty);
        }
    } else {
        let any = Expected::new(var.pos, &Expect::any());
        for (f_mut, f_name) in identifier.fields(var.pos)? {
            env_with_var = env_with_var.insert_var(mutable && f_mut, &f_name, &any);
        }
    };

    if identifier.is_tuple() {
        let tup_exps = identifier_to_tuple(var.pos, &identifier, &env_with_var)?;
        if let Some(ty) = ty {
            let ty_exp = Expected::try_from((ty, &env_with_var.var_mappings))?;
            for tup_exp in &tup_exps {
                constr.add("type and tuple", &ty_exp, tup_exp);
            }
        }
        if let Some(expr) = expr {
            let expr_expt = Expected::try_from((expr, &env_with_var.var_mappings))?;
            for tup_exp in &tup_exps {
                constr.add("tuple and expression", &expr_expt, tup_exp);
            }
        }
    }

    let var_expect = Expected::try_from((var, &env_with_var.var_mappings))?;
    if let Some(ty) = ty {
        let ty_exp = Expected::new(ty.pos, &Type { name: Name::try_from(ty.deref())? });
        constr.add("variable and type", &ty_exp, &var_expect);
    }
    if let Some(expr) = expr {
        let expr_expect = Expected::try_from((expr, &env_with_var.var_mappings))?;
        let msg = format!("variable and expression: `{}`", expr.node);
        constr.add(&msg, &var_expect, &expr_expect);
    }

    Ok(env_with_var)
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
                let msg = format!("'{iden}' is undefined in this scope");
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
        let mut builder = ConstrBuilder::new();
        generate(&ast, &Environment::default(), &Context::default(), &mut builder).unwrap();

        // Ignore then and else branches
        let mut constraints = builder.all_constr()[2].clone();

        let mut popped = HashSet::new();
        while let Some(pop) = constraints.pop_constr() {
            popped.insert(pop);
        }
        assert_eq!(popped.len(), 3);
    }
}
