use std::collections::HashSet;
use std::convert::TryFrom;
use std::ops::Deref;

use itertools::EitherOrBoth::{Both, Left, Right};
use itertools::Itertools;

use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::expected::{Expect, Expected};
use crate::check::constrain::constraint::expected::Expect::*;
use crate::check::constrain::generate::{Constrained, gen_vec, generate};
use crate::check::constrain::generate::collection::constr_col;
use crate::check::constrain::generate::env::Environment;
use crate::check::context::{clss, Context, LookupClass, LookupFunction};
use crate::check::context::arg::FunctionArg;
use crate::check::ident::Identifier;
use crate::check::name::Name;
use crate::check::name::stringname::StringName;
use crate::check::result::{TypeErr, TypeResult};
use crate::common::position::Position;
use crate::parse::ast::{AST, Node};

pub fn gen_call(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    match &ast.node {
        Node::Reassign { left, right } => {
            let identifier = check_reassignable(left)?;
            // Top-level reassign should be defined in env
            let mut errors = vec![];
            for (f_mut, var) in &identifier.fields() {
                if !f_mut {
                    // note that this is mutability of reassign iden, not one in env
                    let msg = format!("Cannot change mutability of {} in reassign", var);
                    errors.push(TypeErr::new(&ast.pos, &msg))
                }

                if let Some(expecteds) = env.get_var(var) {
                    if expecteds.iter().any(|(is_mut, _)| !is_mut) {
                        let msg = format!("{} was declared final, cannot reassign", var);
                        errors.push(TypeErr::new(&ast.pos, &msg))
                    }
                } else {
                    let msg = format!("'{}' is undefined in this scope.", var);
                    errors.push(TypeErr::new(&ast.pos, &msg))
                }
            }
            if !errors.is_empty() {
                return Err(errors);
            }

            constr.add(
                "reassign",
                &Expected::try_from((left, &env.var_mappings))?,
                &Expected::try_from((right, &env.var_mappings))?,
            );
            let (mut constr, env) = generate(right, env, ctx, constr)?;
            generate(left, &env, ctx, &mut constr)
        }
        Node::FunctionCall { name, args } => {
            let f_name = StringName::try_from(name)?;
            let (mut constr, env) = gen_vec(args, env, ctx, constr)?;

            if let Some(functions) = env.get_var(&f_name.name) {
                if !f_name.generics.is_empty() {
                    let msg = "Anonymous function call cannot have generics";
                    return Err(vec![TypeErr::new(&name.pos, msg)]);
                }

                for (_, fun_exp) in functions {
                    let last_pos = args.last().map_or_else(|| name.pos.clone(), |a| a.pos.clone());
                    let args = args
                        .iter()
                        .map(|a| Expected::try_from((a, &env.var_mappings)))
                        .collect::<Result<_, _>>()?;
                    let right = Expected::new(&last_pos, &Function { name: f_name.clone(), args });
                    constr.add("function call", &right, &fun_exp);
                }
            } else {
                // Resort to looking up in Context
                let fun = ctx.function(&f_name, &ast.pos)?;
                constr = call_parameters(ast, &fun.arguments, &None, args, ctx, &constr)?;
                let fun_ret_exp = Expected::new(&ast.pos, &Type { name: fun.ret_ty });
                // entire AST is either fun ret ty or statement
                constr.add(
                    "function call",
                    &Expected::try_from((ast, &env.var_mappings))?,
                    &fun_ret_exp,
                );

                if !fun.raises.is_empty() {
                    if let Some(raises) = &env.raises {
                        let raises_exp = Expected::new(&ast.pos, &Raises { name: fun.raises });
                        constr.add("function call", raises, &raises_exp);
                    } else if !constr.is_top_level() {
                        let msg = format!("Exceptions not covered: {}", &fun.raises);
                        return Err(vec![TypeErr::new(&ast.pos, &msg)]);
                    }
                }
            }

            Ok((constr, env))
        }
        Node::PropertyCall { instance, property } => {
            property_call(instance, property, env, ctx, constr)
        }
        Node::Index { item, range } => {
            let (mut constr, _) = generate(range, env, ctx, constr)?;

            let name = Name::from(&HashSet::from([clss::INT_PRIMITIVE, clss::SLICE]));
            constr.add(
                "index range",
                &Expected::new(&range.pos, &Expect::Type { name }),
                &Expected::try_from((range, &env.var_mappings))?,
            );

            let mut constr = constr_col(item, env, &mut constr)?;
            generate(item, env, ctx, &mut constr)
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Was expecting call")]),
    }
}

fn call_parameters(
    self_ast: &AST,
    possible: &[FunctionArg],
    self_arg: &Option<Expect>,
    args: &[AST],
    ctx: &Context,
    constr: &ConstrBuilder,
) -> Result<ConstrBuilder, Vec<TypeErr>> {
    let mut constr = constr.clone();
    let args = if let Some(self_arg) = self_arg {
        let mut new_args = vec![(self_ast.pos.clone(), self_arg.clone())];
        new_args.append(
            &mut args
                .iter()
                .map(|arg| (arg.pos.clone(), Expression { ast: arg.clone() }))
                .collect(),
        );
        new_args
    } else {
        args.iter().map(|arg| (arg.pos.clone(), Expression { ast: arg.clone() })).collect()
    };

    for either_or_both in possible.iter().zip_longest(args.iter()) {
        match either_or_both {
            Both(fun_arg, (pos, arg)) => {
                let ty = &fun_arg.ty.clone().ok_or_else(|| {
                    TypeErr::new(pos, "Function argument must have type parameters")
                })?;

                let arg_exp = Expected::new(pos, arg);
                let name = ctx.class(ty, pos)?.name();
                constr.add("call parameters", &arg_exp, &Expected::new(pos, &Type { name }))
            }
            Left(fun_arg) if !fun_arg.has_default => {
                let pos = Position::new(&self_ast.pos.end, &self_ast.pos.end);
                let msg = format!("Expected argument: '{}' has no default", fun_arg);
                return Err(vec![TypeErr::new(&pos, &msg)]);
            }
            Right((pos, _)) => return Err(vec![TypeErr::new(pos, "Unexpected argument")]),
            _ => {}
        }
    }

    Ok(constr)
}

fn property_call(
    instance: &AST,
    property: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    match &property.node {
        Node::PropertyCall { instance: inner, property } => {
            let (mut constr, env) = property_call(instance, inner, env, ctx, constr)?;
            property_call(inner, property, &env, ctx, &mut constr)
        }
        Node::Id { lit } => {
            let access = Expected::new(
                &property.pos,
                &Access {
                    entity: Box::new(Expected::try_from((instance, &env.var_mappings))?),
                    name: Box::new(Expected::new(&property.pos, &Field { name: lit.clone() })),
                },
            );
            let instance = Expected::try_from((
                &AST {
                    pos: instance.pos.union(&property.pos),
                    node: Node::PropertyCall {
                        instance: Box::from(instance.clone()),
                        property: Box::from(property.clone()),
                    },
                },
                &env.var_mappings,
            ))?;
            constr.add("call property", &instance, &access);
            Ok((constr.clone(), env.clone()))
        }
        Node::Reassign { left, right } => {
            check_reassignable(left)?;
            let name = match &left.node {
                Node::Id { lit } => lit.clone(),
                _ => {
                    return Err(vec![TypeErr::new(&right.pos, "Expected identifier in reassign.")])
                }
            };

            let left = Expected::new(
                &property.pos,
                &Access {
                    entity: Box::new(Expected::try_from((instance, &env.var_mappings))?),
                    name: Box::new(Expected::new(&property.pos, &Field { name })),
                },
            );

            constr.add(
                "call and reassign",
                &left,
                &Expected::try_from((right, &env.var_mappings))?,
            );
            generate(right, env, ctx, constr)
        }
        Node::FunctionCall { name, args } => {
            let (mut constr, env) = gen_vec(args, env, ctx, constr)?;
            let instance_exp = Expected::try_from((instance, &env.var_mappings))?;
            let mut args_with_self: Vec<Expected> = vec![instance_exp];
            args_with_self.append(
                &mut args
                    .iter()
                    .map(|a| Expected::try_from((a, &env.var_mappings)))
                    .collect::<Result<_, _>>()?,
            );

            let instance_exp = Expected::try_from((
                &AST {
                    pos: instance.pos.union(&property.pos),
                    node: Node::PropertyCall {
                        instance: Box::from(instance.clone()),
                        property: Box::from(property.clone()),
                    },
                },
                &env.var_mappings,
            ))?;
            let access = Expected::new(
                &property.pos,
                &Access {
                    entity: Box::new(Expected::try_from((instance, &env.var_mappings))?),
                    name: Box::new(Expected::new(
                        &property.pos,
                        &Function {
                            name: StringName::try_from(name.deref())?,
                            args: args_with_self,
                        },
                    )),
                },
            );

            constr.add("call class function", &instance_exp, &access);
            Ok((constr, env))
        }

        _ => Err(vec![TypeErr::new(&property.pos, "Expected property call")]),
    }
}

/// Check if AST is something was can be re-assigned to.
///
/// This is true if it is a valid identifier, or a property call which is a identifier.
/// A property call may not be a tuple, however.
fn check_reassignable(ast: &AST) -> TypeResult<Identifier> {
    match &ast.node {
        Node::PropertyCall { property, .. } => {
            let identifier = check_reassignable(property)?;
            if identifier.is_tuple() {
                let msg = format!("Cannot reassign to {}", &ast.node);
                Err(vec![TypeErr::new(&ast.pos, &msg)])
            } else {
                Ok(identifier)
            }
        }
        _ => Identifier::try_from(ast).map_err(|errs| {
            errs.iter()
                .map(|err| {
                    let msg = format!("Cannot reassign to {}: {}", &ast.node, &err.msg);
                    TypeErr::new(&ast.pos, &msg)
                })
                .collect()
        }),
    }
}
