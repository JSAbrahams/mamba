use crate::core::construct::Core;
use crate::desugar::context::Imports;
use crate::desugar::context::State;
use crate::desugar::desugar_result::DesugarResult;
use crate::desugar::node::desugar_node;
use crate::parser::ast::ASTNodePos;

pub fn desugar_vec(
    node_vec: &[ASTNodePos],
    imp: &mut Imports,
    state: &State
) -> DesugarResult<Vec<Core>> {
    let mut result = vec![];
    for node_pos in node_vec {
        result.push(desugar_node(node_pos, imp, state)?)
    }

    Ok(result)
}
