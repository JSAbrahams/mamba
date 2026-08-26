use cranelift_codegen::ir::types;
use cranelift_codegen::ir::Type;

use crate::check::ast::ASTTy;
use crate::check::context::clss;
use crate::check::name::Name;
use crate::common::position::Position;

use crate::backend::cranelift::result::{BackendErr, BackendResult};

/// The Cranelift type a resolved Mamba primitive lowers to.
///
/// Scoped to exactly the primitives the Cranelift backend supports (see `convert/`): `Int`,
/// `Bool`, and `Float`. Anything else (a class, a collection, a union of more than one name,
/// an unresolved type) is out of scope for this backend.
pub fn cranelift_type(ast: &ASTTy) -> BackendResult<Type> {
    let name = ast
        .ty
        .as_ref()
        .ok_or_else(|| BackendErr::new(ast.pos, "Expression has no resolved type"))?;
    cranelift_type_of_name(name, ast.pos)
}

/// As [cranelift_type], but for a [Name] that isn't attached to an [ASTTy] node's own `ty` field
/// -- e.g. a `FunArg`'s declared parameter type, which lives in the `FunArg` variant itself
/// rather than in the surrounding node's resolved type.
pub fn cranelift_type_of_name(name: &Name, pos: Position) -> BackendResult<Type> {
    primitive_name(name)
        .and_then(|name| match name {
            clss::INT => Some(types::I64),
            clss::BOOL => Some(types::I8),
            clss::FLOAT => Some(types::F64),
            _ => None,
        })
        .ok_or_else(|| {
            BackendErr::new(
                pos,
                &format!("The '{name}' type is not supported by the machine-code backend"),
            )
        })
}

/// The single primitive class name a resolved [Name] refers to, if it is exactly one non-generic,
/// non-nullable name -- i.e. not a union of multiple types, and not a generic instantiation like
/// `List[Int]`. Every type this backend supports is shaped this way. Mutability is *not*
/// disqualifying -- it's a property of the binding (and Mamba function arguments are mutable by
/// default), not of the underlying machine representation, which is identical either way.
fn primitive_name(name: &Name) -> Option<&str> {
    let mut names = name.names.iter();
    let true_name = names.next()?;
    if names.next().is_some() {
        return None; // union of more than one type
    }
    if true_name.is_nullable || !true_name.variant.generics.is_empty() {
        return None;
    }
    Some(true_name.variant.name.as_str())
}
