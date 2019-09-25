use crate::parser::ast::{Node, AST};
use crate::type_checker::context::Context;
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::InferResult;
use crate::type_checker::type_result::TypeErr;

pub fn infer_assign(ast: &AST, _: &Environment, _: &Context, _: &State) -> InferResult {
    match &ast.node {
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
