use crate::parser::ast::{Node, AST};
use crate::type_checker::context::Context;
use crate::type_checker::modify::modifications::Modification;
use crate::type_checker::type_result::TypeResult;

pub struct Retry;

impl Retry {
    pub fn new() -> Retry { Retry {} }
}

impl Modification for Retry {
    fn modify(&self, ast: &AST, ctx: &Context) -> TypeResult<AST> {
        match &ast.node {
            Node::Handle { .. } => unimplemented!(),
            _ => self.recursion(ast, ctx)
        }
    }
}
