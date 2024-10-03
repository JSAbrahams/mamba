use crate::generate::ast::node::Core;
use crate::generate::convert::convert_node;
use crate::generate::convert::state::{Imports, State};
use crate::generate::result::GenResult;
use crate::{ASTTy, Context};

pub fn convert_vec(
    node_vec: &[ASTTy],
    imp: &mut Imports,
    state: &State,
    ctx: &Context,
) -> GenResult<Vec<Core>> {
    let mut result = vec![];
    for ast in node_vec {
        result.push(convert_node(ast, imp, state, ctx)?)
    }

    Ok(result)
}
