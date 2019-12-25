use crate::parser::ast::{Node, AST};
use crate::type_checker::context::Context;
use crate::type_checker::modify::modifications::Modification;
use crate::type_checker::type_result::TypeResult;

pub struct Forward;

impl Forward {
    pub fn new() -> Forward { Forward {} }
}

impl Modification for Forward {
    fn modify(&self, ast: &AST, ctx: &Context) -> TypeResult<AST> {
        match &ast.node {
            Node::VariableDef { forward, .. } =>
                if forward.is_empty() {
                    Ok(ast.clone())
                } else {
                    unimplemented!()
                },
            _ => unimplemented!()
        }
    }
}
