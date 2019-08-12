use crate::parser::ast::{ASTNode, ASTNodePos};
use crate::type_checker::type_node::Ty;
use crate::type_checker::type_node::Type;

#[derive(Debug)]
pub struct Field {
    id:       String,
    location: Vec<String>,
    mutable:  bool,
    private:  bool,
    ty:       Type
}

impl Field {
    pub fn new(node_pos: &ASTNodePos) -> Result<Field, String> {
        match &node_pos.node {
            ASTNode::VariableDef { .. } => Ok(Field {
                id:       String::from("my_field"),
                location: vec![],
                mutable:  false,
                private:  false,
                ty:       Type::new(&Ty::Empty)
            }),
            other => Err(format!("Expected field but got {:?}", other))
        }
    }
}
