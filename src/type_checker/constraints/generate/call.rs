use std::collections::HashSet;
use std::convert::TryFrom;
use std::ops::Deref;

use itertools::EitherOrBoth::{Both, Left, Right};
use itertools::Itertools;

use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::cons::Expect::{Expression, Function, HasField, HasFunction,
                                                     Mutable, Type};
use crate::type_checker::constraints::cons::{Constraints, Expect};
use crate::type_checker::constraints::generate::{gen_vec, generate};
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::function_arg::concrete::FunctionArg;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::type_name::TypeName;
use crate::type_checker::type_result::TypeErr;

pub fn gen_call(ast: &AST, env: &Environment, ctx: &Context, constr: &Constraints) -> Constrained {
    match &ast.node {
        Node::Reassign { left, right } => {
            let mutable = Mutable { expect: Box::from(Expression { ast: *left.clone() }) };
            let constr = constr.add(&mutable, &Expression { ast: *right.clone() });
            let (constr, env) = generate(right, env, ctx, &constr)?;
            generate(left, &env, ctx, &constr)
        }
        Node::ConstructorCall { name, args } => {
            let c_name = TypeName::try_from(name.deref())?;
            let constr_args = ctx.lookup(&c_name, &ast.pos)?.constructor_args(&ast.pos)?;

            let self_arg = Some(Type { type_name: c_name });
            let constr = call_parameters(ast, &constr_args, &self_arg, args, constr)?;
            gen_vec(args, env, ctx, &constr)
        }
        Node::FunctionCall { name, args } => {
            let f_name = TypeName::try_from(name.deref())?;
            let f_str_name = f_name.clone().single(&name.pos)?.name(&name.pos)?;
            let (constr, env) = gen_vec(args, env, ctx, constr)?;

            Ok((
                match env.get_var_new(&f_str_name) {
                    Some(function) => {
                        let args = args.iter().map(|arg| Expression { ast: arg.clone() }).collect();
                        constr.add(&function, &Function { name: f_name, args })
                    }
                    None => {
                        // Resort to looking up in Context
                        let possible_fun_args = ctx.lookup_fun_args(&f_name, &ast.pos)?;
                        call_parameters(ast, &possible_fun_args, &None, args, &constr)?
                    }
                },
                env
            ))
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
    args: &Vec<AST>,
    constr: &Constraints
) -> Result<Constraints, Vec<TypeErr>> {
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

    let paired = possible_it.flat_map(|f_args| f_args.into_iter().zip_longest(args.iter()));
    for either_or_both in paired {
        match either_or_both {
            Both(fun_arg, (pos, arg)) => {
                let ty = &fun_arg.ty.as_ref();
                let type_name =
                    ty.ok_or_else(|| TypeErr::new(&pos, "Must have type parameters"))?.clone();
                constr = constr.add(&arg, &Type { type_name })
            }
            Left(fun_arg) if !fun_arg.has_default => {
                let pos = Position::new(&ast.pos.end, &ast.pos.end);
                return Err(vec![TypeErr::new(&pos, "Expected argument")]);
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
    constr: &Constraints
) -> Constrained {
    match &property.node {
        Node::PropertyCall { instance: inner, property } => {
            let (constr, env) = property_call(instance, inner, env, ctx, constr)?;
            property_call(inner, property, &env, ctx, &constr)
        }
        Node::Id { lit } => {
            let instance = Expression { ast: instance.clone() };
            let constr = constr.add(&instance, &HasField { name: lit.clone() });
            Ok((constr, env.clone()))
        }
        Node::Reassign { left, right } => {
            let left_mut = Mutable { expect: Box::from(Expression { ast: *left.clone() }) };
            let constr = constr.add(&left_mut, &Expression { ast: *right.clone() });
            let (constr, env) = generate(right, env, ctx, &constr)?;
            generate(left, &env, ctx, &constr)
        }
        Node::FunctionCall { name, args } => {
            let f_name = TypeName::try_from(name.deref())?;
            let (constr, env) = gen_vec(args, env, ctx, constr)?;

            let args = args.iter().map(|arg| Expression { ast: arg.clone() }).collect();
            let instance = Expression { ast: instance.clone() };
            let constr = constr.add(&instance, &HasFunction { name: f_name, args });

            Ok((constr, env))
        }

        _ => Err(vec![TypeErr::new(&property.pos, "Expected property call")])
    }
}
