use crate::common::position::Position;
use crate::parser::ast::{ASTNode, ASTNodePos};
use crate::type_checker::type_result::TypeErr;

pub fn try_from_id(node_pos: &ASTNodePos) -> Result<String, TypeErr> {
    match &node_pos.node {
        ASTNode::Id { lit } => Ok(lit.clone()),
        _ => Err(TypeErr::new(Position::from(node_pos), "Expected identifier"))
    }
}
