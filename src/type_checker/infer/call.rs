use crate::parser::ast::{Node, AST};
use crate::type_checker::context::Context;
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::InferResult;
use crate::type_checker::type_result::TypeErr;

pub fn infer_call(ast: &AST, _: &Environment, _: &Context, _: &State) -> InferResult {
    match &ast.node {
        Node::FunctionCall { .. } => unimplemented!(),
        Node::PropertyCall { .. } => unimplemented!(),
        _ => Err(vec![TypeErr::new(&ast.pos, "Expected class or class element")])
    }
}
