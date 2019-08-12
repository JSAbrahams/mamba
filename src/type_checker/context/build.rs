use crate::parser::ast::ASTNodePos;
use crate::type_checker::context::context::Context;

impl Context {
    pub fn new(node_pos: &[ASTNodePos]) -> Context {
        Context { interfaces: vec![], classes: vec![] }
    }
}
