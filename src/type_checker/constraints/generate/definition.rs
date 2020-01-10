use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::cons::Constraints;
use crate::type_checker::constraints::cons::Expect::{Expression, Type};
use crate::type_checker::constraints::generate::generate;
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::type_name::TypeName;
use crate::type_checker::type_result::TypeErr;
use std::convert::TryFrom;
use std::ops::Deref;

pub fn gen_def(ast: &AST, env: &Environment, ctx: &Context, constr: &Constraints) -> Constrained {
    match &ast.node {
        Node::FunDef { fun_args, ret_ty, body, .. } => {
            for fun_arg in fun_args {
                match &fun_arg.node {
                    Node::FunArg { .. } => {}
                    _ => return Err(vec![TypeErr::new(&fun_arg.pos, "Expected function argument")])
                }
            }

            match (ret_ty, body) {
                (Some(ret_ty), Some(body)) => {
                    let type_name = TypeName::try_from(ret_ty.deref())?;
                    let constr =
                        constr.add(&Expression { ast: *body.clone() }, &Type { type_name });
                    generate(body, &env, ctx, &constr)
                }
                _ => Ok((constr.clone(), env.clone()))
            }
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected definition")])
    }
}
