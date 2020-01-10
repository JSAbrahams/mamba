use std::collections::HashSet;
use std::convert::TryFrom;
use std::ops::Deref;

use itertools::{EitherOrBoth, Itertools};

use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::cons::Constraints;
use crate::type_checker::constraints::cons::Expect;
use crate::type_checker::constraints::generate::common::gen_vec;
use crate::type_checker::constraints::generate::generate;
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::function_arg::concrete::FunctionArg;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::type_name::TypeName;
use crate::type_checker::type_result::TypeErr;

pub fn generate_call(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &Constraints
) -> Constrained {
    match &ast.node {
        Node::Reassign { left, right } => {
            let constr = constr
                .add(&Expect::Expression { ast: left.deref().clone() }, &Expect::Expression {
                    ast: right.deref().clone()
                });
            let constr = constr
                .add(&Expect::Expression { ast: left.deref().clone() }, &Expect::Mutable {
                    expect: Box::from(Expect::AnyExpression)
                });
            let (constr, env) = generate(right, env, ctx, &constr)?;
            generate(left, &env, ctx, &constr)
        }
        Node::ConstructorCall { name, args } => {
            let type_name = TypeName::try_from(name.deref())?;
            // TODO lookup in environment if not in context
            let possible_constructor_args: HashSet<Vec<FunctionArg>> =
                ctx.lookup(&type_name, &ast.pos)?.constructor_args(&ast.pos)?;

            let constr = fun_args(ast, &possible_constructor_args, args, constr)?;
            let (constr, env) = gen_vec(args, env, ctx, &constr)?;
            generate_call(name, &env, ctx, &constr)
        }
        Node::FunctionCall { name, args } => {
            let type_name = TypeName::try_from(name.deref())?;
            // TODO lookup in environment if not in context
            let possible_fun_args: HashSet<Vec<FunctionArg>> =
                ctx.lookup_fun_args(&type_name, &ast.pos)?;

            let constr = fun_args(ast, &possible_fun_args, args, constr)?;
            let (constr, env) = gen_vec(args, env, ctx, &constr)?;
            generate_call(name, &env, ctx, &constr)
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
    for fun_args in possible {
        for pair in fun_args.iter().zip_longest(args.iter()) {
            match pair {
                EitherOrBoth::Both(fun_arg, arg) =>
                    constr = constr.add(
                        &Expect::Expression { ast: arg.deref().clone() },
                        &Expect::Type {
                            type_name: fun_arg
                                .ty
                                .as_ref()
                                .ok_or_else(|| {
                                    TypeErr::new(&arg.pos, "Functions mut have type parameters")
                                })?
                                .clone()
                        }
                    ),
                EitherOrBoth::Left(fun_arg) if !fun_arg.has_default =>
                    return Err(vec![TypeErr::new(
                        &Position::new(&ast.pos.end, &ast.pos.end),
                        "Expected argument"
                    )]),
                EitherOrBoth::Right(arg) =>
                    return Err(vec![TypeErr::new(&arg.pos, "Unexpected argument")]),
                _ => {}
            }
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
