use std::collections::HashSet;
use std::convert::TryFrom;
use std::ops::Deref;

use itertools::EitherOrBoth::{Both, Left, Right};
use itertools::Itertools;

use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::checker_result::TypeErr;
use crate::type_checker::constraints::constraint::builder::ConstrBuilder;
use crate::type_checker::constraints::constraint::expected::Expect::*;
use crate::type_checker::constraints::constraint::expected::{Expect, Expected};
use crate::type_checker::constraints::generate::{gen_vec, generate};
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::function_arg::concrete::FunctionArg;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::ty_name::TypeName;

pub fn gen_call(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder
) -> Constrained {
    match &ast.node {
        Node::Reassign { left, right } => {
            let l_exp = Expected::from(left);
            let r_exp = Expected::new(&left.pos, &Mutable);
            constr.add(&l_exp, &r_exp);

            let r_exp = Expected::from(right);
            constr.add(&l_exp, &r_exp);

            let (mut constr, env) = generate(right, env, ctx, constr)?;
            generate(left, &env, ctx, &mut constr)
        }
        Node::ConstructorCall { name, args } => {
            let c_name = TypeName::try_from(name.deref())?;
            let constr_args = ctx.lookup(&c_name, &ast.pos)?.constructor_args(&ast.pos)?;
            let self_type = Type { type_name: c_name };

            constr.add(&Expected::new(&ast.pos, &self_type), &Expected::from(ast));

            let self_arg = Some(self_type);
            let constr = call_parameters(ast, &constr_args, &self_arg, args, constr)?;
            gen_vec(args, env, ctx, &constr)
        }
        Node::FunctionCall { name, args } => {
            let f_name = TypeName::try_from(name.deref())?;
            let f_str_name = f_name.clone().single(&name.pos)?.name(&name.pos)?;
            let (mut constr, env) = gen_vec(args, env, ctx, constr)?;

            if let Some(return_types) = env.get_var(&f_str_name) {
                for (_, function) in return_types {
                    let left = Expected::new(&name.pos, &function);

                    let last_pos = args.last().map_or_else(|| name.pos.clone(), |a| a.pos.clone());
                    let args = args.iter().map(Expected::from).collect();
                    let right = Expected::new(&last_pos, &Function { name: f_name.clone(), args });
                    constr.add(&left, &right);
                }
            } else {
                // Resort to looking up in Context
                let possible_fun_args = ctx.lookup_fun_args(&f_name, &ast.pos)?;
                call_parameters(ast, &possible_fun_args, &None, args, &constr)?;
            }

            Ok((constr, env))
        }
        Node::PropertyCall { instance, property } =>
            property_call(instance, property, env, ctx, constr),

        _ => Err(vec![TypeErr::new(&ast.pos, "Was expecting call")])
    }
}

fn call_parameters(
    ast: &AST,
    possible: &HashSet<Vec<FunctionArg>>,
    self_arg: &Option<Expect>,
    args: &[AST],
    constr: &ConstrBuilder
) -> Result<ConstrBuilder, Vec<TypeErr>> {
    let mut constr = constr.clone();
    let possible_it = possible.iter();

    let args = if let Some(self_arg) = self_arg {
        let mut new_args = vec![(ast.pos.clone(), self_arg.clone())];
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

    let paired = possible_it.flat_map(|f_args| f_args.iter().zip_longest(args.iter()));
    for either_or_both in paired {
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
                let pos = Position::new(&ast.pos.end, &ast.pos.end);
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
            let left = Expected::from(instance);
            constr.add(&left, &Expected::new(&property.pos, &HasField { name: lit.clone() }));
            Ok((constr.clone(), env.clone()))
        }
        Node::Reassign { left, right } => {
            let l_exp = Expected::from(left);
            constr.add(&l_exp, &Expected::new(&left.pos, &Mutable));
            constr.add(&l_exp, &Expected::from(right));
            let (mut constr, env) = generate(right, env, ctx, constr)?;
            generate(left, &env, ctx, &mut constr)
        }
        Node::FunctionCall { name, args } => {
            let (mut constr, env) = gen_vec(args, env, ctx, constr)?;
            let last_pos = args.last().map_or_else(|| name.pos.clone(), |a| a.pos.clone());
            let instance_exp = Expected::from(instance);

            let mut args_with_self: Vec<Expected> = vec![instance_exp.clone()];
            args_with_self.append(&mut args.iter().map(Expected::from).collect());

            let f_name = TypeName::try_from(name.deref())?;
            let implements = Implements { type_name: f_name, args: args_with_self };
            let right = Expected::new(&last_pos, &implements);
            constr.add(&instance_exp, &right);
            Ok((constr, env))
        }

        _ => Err(vec![TypeErr::new(&property.pos, "Expected property call")])
    }
}
