use crate::parser::ast::{Node, AST};
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::context::Context;
use crate::type_checker::environment::infer_type::InferType;
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::TypeErr;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::ops::Deref;

pub fn infer_call(ast: &AST, env: &Environment, ctx: &Context, state: &State) -> InferResult {
    match &ast.node {
        Node::FunctionCall { name, args } => {
            let fun_name = TypeName::try_from(name.deref())?;
            let mut args_names = vec![];
            let mut raises = HashSet::new();
            let mut env = env.clone();
            for arg in args {
                let (ty, new_env) = infer(arg, &env, ctx, state)?;
                args_names.push(TypeName::from(&ty.expr_ty(&arg.pos)?));
                env = new_env;
                raises = raises.union(&ty.raises).cloned().collect();
            }

            match env.lookup(&fun_name.clone().single(&name.pos)?.name(&name.pos)?, &name.pos) {
                Ok(expr_ty) => Ok((
                    InferType::from(&expr_ty.anon_fun(&args_names, &ast.pos)?)
                        .union_raises(&raises),
                    env.clone()
                )),
                Err(_) => Ok((ctx.lookup_fun(&fun_name, &args_names, &ast.pos)?, env))
            }
        }
        Node::PropertyCall { .. } => unimplemented!(),
        _ => Err(vec![TypeErr::new(&ast.pos, "Expected class or class element")])
    }
}
