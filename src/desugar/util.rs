use crate::core::construct::Core;
use crate::desugar::node::desugar_node;
use crate::parser::ast::ASTNodePos;

pub fn desugar_vec(node_pos: &[ASTNodePos]) -> Vec<Core> {
    node_pos.iter().map(|node_pos| desugar_node(node_pos)).collect()
}
