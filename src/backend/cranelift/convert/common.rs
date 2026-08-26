use crate::backend::cranelift::result::{BackendErr, BackendResult};
use crate::check::ast::{ASTTy, NodeTy};

/// The identifier an `Id` node holds, e.g. a `FunDef`'s name or a `FunArg`'s bound variable.
pub(super) fn fun_name(id: &ASTTy) -> BackendResult<String> {
    match &id.node {
        NodeTy::Id { lit } => Ok(lit.clone()),
        other => Err(BackendErr::unimplemented(
            id,
            &format!("{other:?} function name"),
        )),
    }
}
