use crate::core::construct::Core;
use crate::desugar::context::Context;
use crate::desugar::context::State;
use crate::desugar::node::desugar_node;
use crate::parser::ast::ASTNodePos;

pub fn desugar_vec(node_pos: &[ASTNodePos], ctx: &Context, state: &State) -> Vec<Core> {
    node_pos.iter().map(|node_pos| desugar_node(node_pos, ctx, state)).collect()
}
