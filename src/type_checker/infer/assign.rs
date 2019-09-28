use crate::parser::ast::{Node, AST};
use crate::type_checker::context::Context;
use crate::type_checker::environment::infer_type::InferType;
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::InferResult;
use crate::type_checker::type_result::TypeErr;

pub fn infer_assign(ast: &AST, env: &Environment, ctx: &Context, state: &State) -> InferResult {
    match &ast.node {
        Node::Id { lit } => Ok((InferType::from(&env.lookup(lit, &ast.pos)?), env.clone())),
        Node::Reassign { .. } => unimplemented!(),
        // TODO use forward and private
        Node::VariableDef { id_maybe_type, expression, .. } => match &id_maybe_type.node {
            // Check whether mutable
            Node::IdType { _type, .. } => match (_type, expression) {
                (Some(_), Some(_)) => unimplemented!(),
                (None, Some(_)) => unimplemented!(),
                (Some(_), None) => unimplemented!(),
                (None, None) => unimplemented!()
            },
            _ => Err(vec![TypeErr::new(&ast.pos, "Expected identifier")])
        },
        Node::FunArg { .. } => unimplemented!(),
        Node::FunDef { .. } => unimplemented!(),

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected variable manipulation")])
    }
}
