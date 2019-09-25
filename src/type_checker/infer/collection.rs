use crate::parser::ast::{Node, AST};
use crate::type_checker::context::Context;
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::InferResult;
use crate::type_checker::type_result::TypeErr;

pub fn infer_coll(ast: &AST, _: &Environment, _: &Context, _: &State) -> InferResult {
    match &ast.node {
        Node::Tuple { .. } => unimplemented!(),
        Node::Set { .. } | Node::List { .. } => unimplemented!(),
        Node::ListBuilder { .. } => unimplemented!(),
        Node::SetBuilder { .. } => unimplemented!(),

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected collection")])
    }
}
