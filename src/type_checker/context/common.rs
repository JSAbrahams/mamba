use crate::parser::ast::{ASTNode, ASTNodePos};
use crate::type_checker::type_result::TypeErr;

pub fn try_from_id(node_pos: &ASTNodePos) -> Result<String, TypeErr> {
    match &node_pos.node {
        ASTNode::Id { lit } => Ok(lit.clone()),
        ASTNode::_Self => Ok(String::from("self")),
        ASTNode::Init => Ok(String::from("init")),
        _ => Err(TypeErr::new(&node_pos.position, "Expected identifier"))
    }
}
