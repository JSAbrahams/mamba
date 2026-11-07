use crate::backend::python::ast::node::PythonCore;
use crate::backend::python::convert::convert_node;
use crate::backend::python::convert::state::{Imports, State};
use crate::backend::python::result::GenResult;
use crate::check::ast::NodeTy;
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

/// Note that a module turns these values into text, which for an `Int` must not be limited in digits.
pub fn turned_into_text(values: &[ASTTy], imp: &mut Imports) {
    if values.iter().any(may_be_int) {
        imp.unbounded_int_text();
    }
}

/// Whether a value may be or hold an `Int`, which only a literal of another type rules out.
///
/// Its type cannot tell, since a value given to `print` is recorded as the `Str` it is turned into.
fn may_be_int(value: &ASTTy) -> bool {
    !matches!(
        value.node,
        NodeTy::Str { .. } | NodeTy::Real { .. } | NodeTy::Bool { .. } | NodeTy::Undefined
    )
}
