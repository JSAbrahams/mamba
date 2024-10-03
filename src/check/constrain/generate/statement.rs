use std::collections::HashSet;
use std::iter::FromIterator;

use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::generate::env::Environment;
use crate::check::constrain::generate::{generate, Constrained};
use crate::check::context::clss::HasParent;
use crate::check::context::{Context, LookupClass};
use crate::check::name::true_name::TrueName;
use crate::check::result::TypeErr;
use crate::common::position::Position;
use crate::parse::ast::{Node, AST};

pub fn gen_stmt(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    match &ast.node {
        Node::Raise { error } => match &error.node {
            Node::FunctionCall { name, .. } => {
                if let Node::Id { lit } = &name.node {
                    let raises = HashSet::from_iter([TrueName::from(lit.as_str())]);
                    check_raises_caught(&raises, env, ctx, ast.pos)?;
                    Ok(env.clone())
                } else {
                    Err(vec![TypeErr::new(
                        name.pos,
                        &format!("Malformed raise: {}", name.node),
                    )])
                }
            }
            _ => Err(vec![TypeErr::new(
                error.pos,
                &format!("Malformed raise: {}", error.node),
            )]),
        },
        Node::ReturnEmpty => {
            if let Some(exp) = &env.return_type {
                let msg = format!("Empty return in function which returns '{exp}'");
                Err(vec![TypeErr::new(ast.pos, &msg)])
            } else if !env.in_fun {
                Err(vec![TypeErr::new(ast.pos, "Return outside function")])
            } else {
                Ok(env.clone())
            }
        }
        Node::Return { expr } => {
            if let Some(expected_ret_ty) = &env.return_type {
                generate(expr, env, ctx, constr)?;
                constr.add("return", expected_ret_ty, &Expected::from(expr), env);
                Ok(env.clone())
            } else if !env.in_fun {
                Err(vec![TypeErr::new(ast.pos, "Return outside function")])
            } else {
                Err(vec![TypeErr::new(
                    ast.pos,
                    "Return outside function with return type",
                )])
            }
        }
        _ => Err(vec![TypeErr::new(ast.pos, "Expected statement")]),
    }
}

/// Check whether a set of raises is properly dealt with if in function body.
///
/// Makes use of the [Environment::raises_caught] field.
/// For each raises, checks whether it or a parent of it is caught.
/// If we are a top-level script, we perform no check as raises do not need to be caught here.
pub fn check_raises_caught(
    raises: &HashSet<TrueName>,
    env: &Environment,
    ctx: &Context,
    pos: Position,
) -> Constrained<()> {
    if env.in_fun {
        let errs: Vec<TypeErr> = raises
            .iter()
            .filter(|raise_name| {
                !if let Ok(raise_class) = ctx.class(*raise_name, pos) {
                    env.raises_caught.iter().any(|env_raise| {
                        if let Ok(result) = raise_class.has_parent(env_raise, ctx, pos) {
                            result
                        } else {
                            false
                        }
                    })
                } else {
                    false
                }
            })
            .map(|n| TypeErr::new(pos, &format!("Exception not caught: {n}")))
            .collect();

        if !errs.is_empty() {
            return Err(errs);
        }
    }

    Ok(())
}
