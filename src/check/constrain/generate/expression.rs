use std::convert::TryFrom;

use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::expected::Expect::Type;
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::constraint::Constraint;
use crate::check::constrain::generate::definition::{constrain_args, id_from_var};
use crate::check::constrain::generate::env::Environment;
use crate::check::constrain::generate::operation::gen_primitive;
use crate::check::constrain::generate::{generate, Constrained};
use crate::check::context::clss::BOOL;
use crate::check::context::Context;
use crate::check::name::true_name::TrueName;
use crate::check::name::{Name, TupleCallable};
use crate::check::result::{TypeErr, TypeResult};
use crate::parse::ast::{Node, OptAST, AST};

pub fn gen_expr(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    match &ast.node {
        Node::AnonFun { args, body } => {
            let arg_types = arg_types(args)?;
            let anon_env = constrain_args(args, &env.is_def_mode(true), ctx, constr)?;
            generate(body, &anon_env, ctx, constr)?;

            constrain_fun_type(ast, body, &arg_types, env, &anon_env, constr);
            Ok(env.clone())
        }
        Node::ExpressionType { expr, mutable, ty } => {
            match_id(expr, ty, *mutable, env, ctx, constr)
        }
        Node::Id { .. } => match_id(ast, &None, false, env, ctx, constr),
        Node::Question { left, right } => {
            constr.add(
                "question",
                &Expected::from(left),
                &Expected::none(left.pos),
                env,
            );

            generate(left, env, ctx, constr)?;
            generate(right, env, ctx, constr)?;
            Ok(env.clone())
        }
        Node::Pass => {
            if let Some(expected_ret_ty) = &env.return_type {
                constr.add("pass", &Expected::none(ast.pos), expected_ret_ty, env);
                Ok(env.clone())
            } else {
                Ok(env.clone())
            }
        }

        _ => Err(vec![TypeErr::new(ast.pos, "Expected an expression")]),
    }
}

/// The type of each argument of an anonymous function.
///
/// As in a function signature, every argument must have one.
/// The return type is inferred, since the body is always an expression.
fn arg_types(args: &[AST]) -> TypeResult<Vec<Name>> {
    args.iter().map(arg_type).collect()
}

fn arg_type(arg: &AST) -> TypeResult<Name> {
    let Node::FunArg { ty: Some(ty), .. } = &arg.node else {
        let msg = "An argument of an anonymous function must have a type";
        return Err(vec![TypeErr::new(arg.pos, msg)]);
    };
    Name::try_from(ty)
}

/// Type an anonymous function as a function from `args` to whatever its body gives.
///
/// That lets its arity and argument types be checked wherever a function is expected.
/// It runs after the body is generated, which can rename the variables the function reads.
/// The function then reads as it does in the constraints around it.
fn constrain_fun_type(
    ast: &AST,
    body: &AST,
    args: &[Name],
    env: &Environment,
    anon_env: &Environment,
    constr: &mut ConstrBuilder,
) {
    let ret = constr.temp_name();
    let ret_exp = Expected::new(body.pos, &Type { name: ret.clone() });
    constr.add(
        "anonymous function body",
        &ret_exp,
        &Expected::from(body),
        anon_env,
    );

    let fun = Expected::new(
        ast.pos,
        &Type {
            name: Name::from(&TrueName::callable(args, &ret)),
        },
    );
    constr.add("anonymous function", &fun, &Expected::from(ast), env);
}

fn match_id(
    ast: &AST,
    ty: &OptAST,
    mutable: bool,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    match &ast.node {
        Node::Id { lit } => {
            // TODO use pre-defined "Undefined", "True" and "False" in context
            if lit.as_str() == "None" {
                constr.add_constr(
                    &Constraint::undefined("undefined", &Expected::from(ast)),
                    env,
                );
                Ok(env.clone())
            } else if lit.as_str() == "True" || lit.as_str() == "False" {
                constr.add_constr(&Constraint::truthy("bool", &Expected::from(ast)), env);
                gen_primitive(ast, BOOL, env, constr)
            } else if env.is_def_mode {
                let ty = if let Some(ty) = ty {
                    Some(Name::try_from(ty)?)
                } else {
                    None
                };
                id_from_var(ast, &ty, &None, mutable, ctx, constr, env)
            } else if env.is_destruct_mode {
                Ok(env.remove_var(lit))
            } else if env.get_var(lit, &constr.var_mapping).is_some() {
                if env.in_pure && env.outer_mut.contains(lit) {
                    // Its value can change between two calls with the same arguments.
                    let msg = format!(
                        "A pure function cannot read '{lit}', which is a 'mut' variable defined outside it"
                    );
                    return Err(vec![TypeErr::new(ast.pos, &msg)]);
                }
                Ok(env.clone())
            } else {
                Err(vec![TypeErr::new(
                    ast.pos,
                    &format!("Undefined variable: {lit}"),
                )])
            }
        }
        _ => generate(ast, env, ctx, constr),
    }
}
