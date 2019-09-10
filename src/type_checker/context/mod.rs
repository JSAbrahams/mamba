use std::convert::TryFrom;

use crate::parser::ast::Node;
use crate::type_checker::context::class::Type;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use crate::type_checker::CheckInput;

pub mod class;
pub mod field;
pub mod function;
pub mod function_arg;
pub mod type_name;

pub mod environment;

mod common;

/// A context stores all information of all identified types of the current
/// application.
///
/// Functions and fields are also stored alongside identified classes such that
/// we can also check usage of top-level fields and functions.
#[derive(Debug)]
pub struct Context {
    types: Vec<Type>
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

impl Context {
    pub fn lookup(&self, _: &TypeName) -> TypeResult { unimplemented!() }
}

impl TryFrom<&[CheckInput]> for Context {
    type Error = Vec<TypeErr>;

    fn try_from(files: &[CheckInput]) -> Result<Self, Self::Error> {
        let mut results = vec![];
        files.iter().for_each(|(file, ..)| match &file.node {
            Node::File { pure, modules, .. } => modules
                .iter()
                .map(|module| match &module.node {
                    Node::Class { .. } | Node::TypeDef { .. } =>
                        results.push(Type::try_from(module).map(|ty| ty.all_pure(*pure))),
                    _ => {}
                })
                .collect(),
            _ => results.push(Err(vec![TypeErr::new(&file.pos, "Expected file")]))
        });

        let (types, type_errs): (Vec<_>, Vec<_>) = results.into_iter().partition(Result::is_ok);

        if !type_errs.is_empty() {
            Err(type_errs.into_iter().map(Result::unwrap_err).flatten().collect())
        } else {
            Ok(Context { types: types.into_iter().map(Result::unwrap).collect() })
        }
    }
}
