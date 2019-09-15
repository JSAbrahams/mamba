use crate::parser::ast::{Node, AST};
use crate::type_checker::context::Context;
use crate::type_checker::environment::expression_type::ExpressionType;
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::TypeErr;

pub fn infer_boolean_operation(
    ast: &Box<AST>,
    env: &Environment,
    ctx: &Context,
    state: &State
) -> InferResult {
    match &ast.node {
        Node::Is { .. } => unimplemented!(),
        Node::IsN { .. } => unimplemented!(),
        Node::Neq { .. } => unimplemented!(),
        Node::IsA { .. } => unimplemented!(),
        Node::IsNA { .. } => unimplemented!(),
        Node::Not { .. } => unimplemented!(),
        Node::And { .. } => unimplemented!(),
        Node::Or { .. } => unimplemented!(),

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected boolean operation")])
    }
}
