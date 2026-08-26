use crate::backend::python::ast::node::PythonCore;
use crate::backend::python::convert::convert_node;
use crate::backend::python::convert::state::{Imports, State};
use crate::backend::python::result::GenResult;
use crate::{ASTTy, Context};

pub fn convert_vec(
    node_vec: &[ASTTy],
    imp: &mut Imports,
    state: &State,
    ctx: &Context,
) -> GenResult<Vec<PythonCore>> {
    let mut result = vec![];
    for ast in node_vec {
        result.push(convert_node(ast, imp, state, ctx)?)
    }

    Ok(result)
}
