use std::collections::HashSet;
use std::convert::TryFrom;
use std::iter::FromIterator;

use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::generate::{Constrained, generate};
use crate::check::constrain::generate::env::Environment;
use crate::check::context::Context;
use crate::check::name::true_name::TrueName;
use crate::check::result::TypeErr;
use crate::common::position::Position;
use crate::parse::ast::{AST, Node};

pub fn gen_stmt(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    match &ast.node {
        Node::Raise { error } => match &error.node {
            Node::FunctionCall { name, .. } => if let Node::Id { lit } = &name.node {
                let raises = HashSet::from_iter([TrueName::from(lit.as_str())]);
                check_raise(constr, &raises, env, ast.pos)?;
                Ok((constr.clone(), env.clone()))
            } else {
                Err(vec![TypeErr::new(name.pos, &format!("Malformed raise: {}", name.node))])
            }
            _ => Err(vec![TypeErr::new(error.pos, &format!("Malformed raise: {}", error.node))])
        }
        Node::ReturnEmpty => {
            if let Some(exp) = &env.return_type {
                let msg = format!("Empty return in function which returns {}", exp);
                Err(vec![TypeErr::new(ast.pos, &msg)])
            } else {
                Ok((constr.clone(), env.clone()))
            }
        }
        Node::Return { expr } => {
            if let Some(expected_ret_ty) = &env.return_type {
                let (mut constr, env) = generate(expr, env, ctx, constr)?;
                constr.add(
                    "return",
                    expected_ret_ty,
                    &Expected::try_from((expr, &env.var_mappings))?,
                );
                Ok((constr, env))
            } else {
                Err(vec![TypeErr::new(ast.pos, "Return outside function with return type")])
            }
        }
        _ => Err(vec![TypeErr::new(ast.pos, "Expected statement")]),
    }
}

/// Check whether a set of raises is properly dealt with.
///
/// Makes use of the [Environment::raises_caught] field.
/// If we are a top-level script, we perform no check as raises do not need to be caught here.
pub fn check_raise(constr: &ConstrBuilder, raises: &HashSet<TrueName>, env: &Environment, pos: Position) -> Constrained<()> {
    if !constr.is_top_level() {
        let errs: Vec<TypeErr> = raises.iter()
            .filter(|n| !env.raises_caught.contains(n))
            .map(|n| TypeErr::new(pos, &format!("Exception not caught: {}", n)))
            .collect();

        if !errs.is_empty() { return Err(errs); }
    }
    Ok(())
}
