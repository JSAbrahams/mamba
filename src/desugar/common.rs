use crate::core::construct::Core;
use crate::desugar::desugar_result::DesugarResult;
use crate::desugar::node::desugar_node;
use crate::desugar::state::Imports;
use crate::desugar::state::State;
use crate::parser::ast::AST;

pub fn desugar_vec(node_vec: &[AST], imp: &mut Imports, state: &State) -> DesugarResult<Vec<Core>> {
    let mut result = vec![];
    for ast in node_vec {
        result.push(desugar_node(ast, imp, state)?)
    }

    Ok(result)
}

pub fn desugar_stmts(
    node_vec: &[AST],
    imp: &mut Imports,
    state: &State
) -> DesugarResult<Vec<Core>> {
    let mut result = vec![];
    for (i, ast) in node_vec.iter().enumerate() {
        if i == node_vec.len() - 1 {
            // only force the last node to be a return or expression if applicable
            result.push(desugar_node(ast, imp, state)?)
        } else {
            result.push(desugar_node(ast, imp, &state.assign_to(None))?)
        }
    }

    Ok(result)
}
