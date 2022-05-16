use crate::generate::ast::node::Core;
use crate::generate::convert::convert_node;
use crate::generate::convert::state::{Imports, State};
use crate::generate::result::GenResult;
use crate::parse::ast::AST;

pub fn convert_vec(node_vec: &[AST], imp: &mut Imports, state: &State) -> GenResult<Vec<Core>> {
    let mut result = vec![];
    for ast in node_vec {
        result.push(convert_node(ast, imp, state)?)
    }

    Ok(result)
}

pub fn convert_stmts(
    node_vec: &[AST],
    imp: &mut Imports,
    state: &State,
) -> GenResult<Vec<Core>> {
    let mut result = vec![];
    for (i, ast) in node_vec.iter().enumerate() {
        if i == node_vec.len() - 1 {
            // only force the last node to be a return or expression if applicable
            result.push(convert_node(ast, imp, state)?)
        } else {
            result.push(convert_node(ast, imp, &state.assign_to(None))?)
        }
    }

    Ok(result)
}
