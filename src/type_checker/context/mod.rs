use crate::parser::ast::ASTNodePos;
use crate::type_checker::context::field::Field;
use crate::type_checker::context::function::Function;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::type_node::Type;
use crate::type_checker::type_result::{TypeErr, TypeResult};

pub mod class;
pub mod field;
pub mod function;
pub mod type_name;

mod common;

/// A context stores all information of all identified types of the current
/// application.
///
/// Functions and fields are also stored alongside identified classes such that
/// we can also check usage of top-level fields and functions.
#[derive(Debug)]
pub struct Context {
    types:     Vec<Type>,
    functions: Vec<Function>,
    fields:    Vec<Field>
}

pub trait ReturnType {
    /// Set return type.
    ///
    /// Mainly for use during the type inference stage.
    ///
    /// # Failures
    ///
    /// If function already has a return type, and the given type is not equal
    /// to said type, a [TypeErr](crate::type_checker::type_result::TypeErr)
    /// is returned.
    fn with_return_type_name(self, ty: TypeName) -> Result<Self, TypeErr>
    where
        Self: Sized;

    /// Get return type.
    ///
    /// This function should be used after the type inference stage.
    ///
    /// # Failures
    ///
    /// Fail if there is no return type. This can happen if either there was no
    /// return type in the signature, or the return type was not set during
    /// the type inference stage (unable to derive return type).
    fn get_return_type_name(&self) -> Result<TypeName, TypeErr>;
}

pub fn build_context(_: &[ASTNodePos]) -> TypeResult<Context> {
    let types = vec![];
    let functions = vec![];
    let fields = vec![];

    Ok(Context { types, functions, fields })
}
