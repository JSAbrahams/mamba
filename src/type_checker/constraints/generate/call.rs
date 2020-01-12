use std::collections::HashSet;
use std::convert::TryFrom;
use std::ops::Deref;

use itertools::EitherOrBoth::{Both, Left, Right};
use itertools::Itertools;

use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::cons::Constraints;
use crate::type_checker::constraints::cons::Expect::{Expression, HasField, Implements, Mutable,
                                                     Type};
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
            // TODO lookup in environment if not in context
            let constr_args = ctx.lookup(&c_name, &ast.pos)?.constructor_args(&ast.pos)?;
            let constr = fun_args(ast, &constr_args, args, constr)?;
            let (constr, env) = gen_vec(args, env, ctx, &constr)?;
            gen_call(name, &env, ctx, &constr)
        }
        Node::FunctionCall { name, args } => {
            let f_name = TypeName::try_from(name.deref())?;
            let possible_fun_args = ctx.lookup_fun_args(&f_name, &ast.pos)?;
            let constr = fun_args(ast, &possible_fun_args, args, constr)?;
            let (constr, env) = gen_vec(args, env, ctx, &constr)?;
            gen_call(name, &env, ctx, &constr)
        }
        Node::PropertyCall { instance, property } =>
            property_call(instance, property, env, ctx, constr),

        _ => Err(vec![TypeErr::new(&ast.pos, "Was expecting call")])
    }
}

fn fun_args(
    ast: &AST,
    possible: &HashSet<Vec<FunctionArg>>,
    args: &Vec<AST>,
    constr: &Constraints
) -> Result<Constraints, Vec<TypeErr>> {
    let mut constr = constr.clone();
    let possible_it = possible.iter();
    let pair_it = possible_it.flat_map(|f_args| f_args.into_iter().zip_longest(args.iter()));

    for either_or_both in pair_it {
        match either_or_both {
            Both(fun_arg, arg) => {
                let ty = &fun_arg.ty.as_ref();
                let type_name =
                    ty.ok_or_else(|| TypeErr::new(&arg.pos, "Must have type parameters"))?.clone();
                constr = constr.add(&Expression { ast: arg.clone() }, &Type { type_name })
            }
            Left(fun_arg) if !fun_arg.has_default => {
                let pos = Position::new(&ast.pos.end, &ast.pos.end);
                return Err(vec![TypeErr::new(&pos, "Expected argument")]);
            }
            Right(arg) => return Err(vec![TypeErr::new(&arg.pos, "Unexpected argument")]),
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
            let property = Expression { ast: property.clone() };
            let constr = constr.add(&property, &HasField { name: lit.clone() });
            Ok((constr, env.clone()))
        }
        Node::Reassign { left, right } => {
            let left_mut = Mutable { expect: Box::from(Expression { ast: *left.clone() }) };
            let constr = constr.add(&left_mut, &Expression { ast: *right.clone() });
            let (constr, env) = generate(right, env, ctx, &constr)?;
            generate(left, &env, ctx, &constr)
        }
        Node::FunctionCall { name, args } => {
            let type_name = TypeName::try_from(name.deref())?;
            let arg_exp = args.iter().map(|arg| Expression { ast: arg.clone() }).collect();

            let property = Expression { ast: property.clone() };
            let constr = constr.add(&property, &Implements { type_name, args: arg_exp });
            let (constr, env) = gen_vec(args, env, ctx, &constr)?;
            generate(name, &env, ctx, &constr)
        }

        _ => Err(vec![TypeErr::new(&property.pos, "Expected property call")])
    }
}
