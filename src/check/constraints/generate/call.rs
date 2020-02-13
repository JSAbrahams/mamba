use std::convert::TryFrom;
use std::ops::Deref;

use itertools::EitherOrBoth::{Both, Left, Right};
use itertools::Itertools;

use crate::check::checker_result::{TypeErr, TypeResult};
use crate::check::constraints::constraint::builder::ConstrBuilder;
use crate::check::constraints::constraint::expected::Expect::*;
use crate::check::constraints::constraint::expected::{Expect, Expected};
use crate::check::constraints::generate::{gen_vec, generate};
use crate::check::constraints::Constrained;
use crate::check::context::function_arg::concrete::FunctionArg;
use crate::check::context::Context;
use crate::check::environment::name::Identifier;
use crate::check::environment::Environment;
use crate::check::ty_name::TypeName;
use crate::common::delimit::comma_delimited;
use crate::common::position::Position;
use crate::parse::ast::{Node, AST};

pub fn gen_call(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder
) -> Constrained {
    match &ast.node {
        Node::Reassign { left, right } => {
            check_reassignable(left)?;
            constr.add(&Expected::from(left), &Expected::from(right));
            let (mut constr, env) = generate(right, env, ctx, constr)?;
            generate(left, &env, ctx, &mut constr)
        }
        Node::ConstructorCall { name, args } => {
            let c_name = TypeName::try_from(name.deref())?;
            let possible_args = ctx.lookup(&c_name, &ast.pos)?.constructor_args(&ast.pos)?;
            let self_type = Type { type_name: c_name };

            constr.add(&Expected::new(&ast.pos, &self_type), &Expected::from(ast));

            let self_arg = Some(self_type);
            let mut constr = constr.clone();
            for constr_args in possible_args {
                constr = call_parameters(ast, &constr_args, &self_arg, args, &constr)?;
            }
            gen_vec(args, env, ctx, &constr)
        }
        Node::FunctionCall { name, args } => {
            let f_name = TypeName::try_from(name.deref())?;
            let f_str_name = f_name.clone().single(&name.pos)?.name(&name.pos)?;
            let (mut constr, env) = gen_vec(args, env, ctx, constr)?;

            if let Some(functions) = env.get_var(&f_str_name) {
                for (_, fun_exp) in functions {
                    let last_pos = args.last().map_or_else(|| name.pos.clone(), |a| a.pos.clone());
                    let args = args.iter().map(Expected::from).collect();
                    let right = Expected::new(&last_pos, &Function { name: f_name.clone(), args });
                    constr.add(&right, &fun_exp);
                }
            } else {
                // Resort to looking up in Context
                let possible_fun = ctx.lookup_function(&f_name, &ast.pos)?;
                for fun in possible_fun {
                    constr = call_parameters(ast, &fun.arguments, &None, args, &constr)?;
                    let fun_ret_exp = if let Some(type_name) = fun.ty() {
                        Expected::new(&ast.pos, &Type { type_name })
                    } else {
                        Expected::new(&ast.pos, &Statement)
                    };
                    // entire AST is either fun ret ty or statement
                    constr.add(&Expected::from(ast), &fun_ret_exp);

                    if !fun.raises.is_empty() {
                        if let Some(raises) = &env.raises {
                            let raises_exp =
                                Raises { raises: fun.raises.iter().map(TypeName::from).collect() };
                            let raises_exp = Expected::new(&ast.pos, &raises_exp);
                            constr.add(&raises, &raises_exp);
                        } else if !constr.is_top_level() {
                            let msg =
                                format!("Exceptions not covered: {}", comma_delimited(&fun.raises));
                            return Err(vec![TypeErr::new(&ast.pos, &msg)]);
                        }
                    }
                }
            }

            Ok((constr, env))
        }
        Node::PropertyCall { instance, property } =>
            property_call(instance, property, env, ctx, constr),

        _ => Err(vec![TypeErr::new(&ast.pos, "Was expecting call")])
    }
}

fn call_parameters(
    self_ast: &AST,
    possible: &Vec<FunctionArg>,
    self_arg: &Option<Expect>,
    args: &[AST],
    constr: &ConstrBuilder
) -> Result<ConstrBuilder, Vec<TypeErr>> {
    let mut constr = constr.clone();
    let args = if let Some(self_arg) = self_arg {
        let mut new_args = vec![(self_ast.pos.clone(), self_arg.clone())];
        new_args.append(
            &mut args
                .iter()
                .map(|arg| (arg.pos.clone(), Expression { ast: arg.clone() }))
                .collect()
        );
        new_args
    } else {
        args.iter().map(|arg| (arg.pos.clone(), Expression { ast: arg.clone() })).collect()
    };

    for either_or_both in possible.iter().zip_longest(args.iter()) {
        match either_or_both {
            Both(fun_arg, (pos, arg)) => {
                let ty = &fun_arg.ty.as_ref();
                let type_name = ty
                    .ok_or_else(|| {
                        TypeErr::new(&pos, "Function argument must have type parameters")
                    })?
                    .clone();

                let left = Expected::new(&pos, &arg);
                constr.add(&left, &Expected::new(&pos, &Type { type_name }))
            }
            Left(fun_arg) if !fun_arg.has_default => {
                let pos = Position::new(&self_ast.pos.end, &self_ast.pos.end);
                return Err(vec![TypeErr::new(&pos, "Expected argument: no default")]);
            }
            Right((pos, _)) => return Err(vec![TypeErr::new(&pos, "Unexpected argument")]),
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
    constr: &mut ConstrBuilder
) -> Constrained {
    match &property.node {
        Node::PropertyCall { instance: inner, property } => {
            let (mut constr, env) = property_call(instance, inner, env, ctx, constr)?;
            property_call(inner, property, &env, ctx, &mut constr)
        }
        Node::Id { lit } => {
            let access = Expected::new(&property.pos, &Access {
                entity: Box::new(Expected::from(instance)),
                name:   Box::new(Expected::new(&property.pos, &Field { name: lit.clone() }))
            });
            let instance = Expected::from(&AST {
                pos:  instance.pos.union(&property.pos),
                node: Node::PropertyCall {
                    instance: Box::from(instance.clone()),
                    property: Box::from(property.clone())
                }
            });
            constr.add(&instance, &access);
            Ok((constr.clone(), env.clone()))
        }
        Node::Reassign { left, right } => {
            check_reassignable(left)?;
            let left = AST {
                pos:  left.pos.clone(),
                node: Node::PropertyCall {
                    instance: Box::from(instance.clone()),
                    property: Box::from(AST { pos: left.pos.clone(), node: left.clone().node })
                }
            };
            constr.add(&Expected::from(&left), &Expected::from(right));
            generate(right, env, ctx, constr)
        }
        Node::FunctionCall { name, args } => {
            let (mut constr, env) = gen_vec(args, env, ctx, constr)?;
            let instance_exp = Expected::from(instance);
            let mut args_with_self: Vec<Expected> = vec![instance_exp.clone()];
            args_with_self.append(&mut args.iter().map(Expected::from).collect());

            let instance_exp = Expected::from(&AST {
                pos:  instance.pos.union(&property.pos),
                node: Node::PropertyCall {
                    instance: Box::from(instance.clone()),
                    property: Box::from(property.clone())
                }
            });
            let access = Expected::new(&property.pos, &Access {
                entity: Box::new(Expected::from(instance)),
                name:   Box::new(Expected::new(&property.pos, &Function {
                    name: TypeName::try_from(name.deref())?,
                    args: args_with_self
                }))
            });

            constr.add(&instance_exp, &access);
            Ok((constr, env))
        }

        _ => Err(vec![TypeErr::new(&property.pos, "Expected property call")])
    }
}

/// Check if AST is reassignable: is reassignable if valid identifier.
fn check_reassignable(ast: &AST) -> TypeResult<()> {
    match &ast.node {
        Node::PropertyCall { property, .. } => check_reassignable(property),
        _ =>
            if Identifier::try_from(ast).is_ok() {
                Ok(())
            } else {
                let msg = format!("Cannot reassign to {}", &ast.node);
                Err(vec![TypeErr::new(&ast.pos, &msg)])
            },
    }
}
