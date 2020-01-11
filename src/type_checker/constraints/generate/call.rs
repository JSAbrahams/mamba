use std::collections::HashSet;
use std::convert::TryFrom;
use std::ops::Deref;

use itertools::{EitherOrBoth, Itertools};

use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::cons::Constraints;
use crate::type_checker::constraints::cons::Expect::{AnyExpr, Expression, Mutable, Type};
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
            let constr = constr
                .add(&Expression { ast: *left.clone() }, &Expression { ast: *right.clone() })
                .add(&Expression { ast: *left.clone() }, &Mutable { expect: Box::from(AnyExpr) });
            let (constr, env) = generate(right, env, ctx, &constr)?;
            generate(left, &env, ctx, &constr)
        }
        Node::ConstructorCall { name, args } => {
            let c_name = TypeName::try_from(name.deref())?;
            // TODO lookup in environment if not in context
            let possible_constr_args = ctx.lookup(&c_name, &ast.pos)?.constructor_args(&ast.pos)?;
            let constr = fun_args(ast, &possible_constr_args, args, constr)?;
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

    for pair in pair_it {
        match pair {
            EitherOrBoth::Both(fun_arg, arg) => {
                let ty = &fun_arg.ty.as_ref();
                let type_name =
                    ty.ok_or_else(|| TypeErr::new(&arg.pos, "Must have type parameters"))?.clone();
                constr = constr.add(&Expression { ast: arg.clone() }, &Type { type_name })
            }
            EitherOrBoth::Left(fun_arg) if !fun_arg.has_default => {
                let pos = Position::new(&ast.pos.end, &ast.pos.end);
                return Err(vec![TypeErr::new(&pos, "Expected argument")]);
            }
            EitherOrBoth::Right(arg) =>
                return Err(vec![TypeErr::new(&arg.pos, "Unexpected argument")]),
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
    unimplemented!()
}
