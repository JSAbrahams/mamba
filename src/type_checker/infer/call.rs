use crate::parser::ast::{Node, AST};
use crate::type_checker::context::Context;
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::InferResult;
use crate::type_checker::type_result::TypeErr;

pub fn infer_call(ast: &AST, env: &Environment, ctx: &Context, state: &State) -> InferResult {
    match &ast.node {
        Node::FunctionCall { name, args } => unimplemented!(),
        Node::PropertyCall { instance, property } => unimplemented!(),

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected class or class element")])
    }
}
