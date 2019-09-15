use crate::parser::ast::{Node, AST};
use crate::type_checker::context::Context;
use crate::type_checker::environment::expression_type::ExpressionType;
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::TypeErr;

pub fn infer_collection(
    ast: &Box<AST>,
    env: &Environment,
    ctx: &Context,
    state: &State
) -> InferResult {
    match &ast.node {
        Node::List { .. } => unimplemented!(),
        Node::ListBuilder { .. } => unimplemented!(),
        Node::Set { .. } => unimplemented!(),
        Node::SetBuilder { .. } => unimplemented!(),
        Node::Tuple { .. } => unimplemented!(),

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected collection")])
    }
}
