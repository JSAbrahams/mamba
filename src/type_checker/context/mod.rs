use std::convert::TryFrom;

use crate::common::position::Position;
use crate::type_checker::context::concrete::Type;
use crate::type_checker::context::generic::field::GenericField;
use crate::type_checker::context::generic::function::GenericFunction;
use crate::type_checker::context::generic::generics;
use crate::type_checker::context::generic::ty::GenericType;
use crate::type_checker::context::generic::type_name::GenericTypeName;
use crate::type_checker::context::python::python_files;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use crate::type_checker::CheckInput;
use std::collections::HashMap;
use std::path::PathBuf;

pub mod concrete;
pub mod generic;

mod python;

// TODO make sets instead of vectors

/// A context stores all information of all identified types of the current
/// application.
///
/// Functions and fields are also stored alongside identified classes such that
/// we can also check usage of top-level fields and functions.
#[derive(Debug)]
pub struct Context {
    pub types: Vec<GenericType>,
    functions: Vec<GenericFunction>,
    fields:    Vec<GenericField>
}

impl Context {
    // TODO rework system for generics
    fn lookup_direct(
        &self,
        name: &str,
        generics: &[GenericTypeName],
        pos: &Position
    ) -> Result<Type, TypeErr> {
        let generic_type = self
            .types
            .iter()
            .find(|ty| ty.name.as_str() == name)
            .ok_or(TypeErr::new(pos, "Cannot find type"))?;

        Type::try_from(generic_type, &HashMap::new(), pos)
    }

    pub fn lookup(&self, type_name: &GenericTypeName, pos: &Position) -> Result<Type, TypeErr> {
        match type_name {
            GenericTypeName::Single { lit, generics } => self.lookup_direct(lit, generics, pos),
            GenericTypeName::Fun { .. } => unimplemented!(),
            GenericTypeName::Tuple { .. } => unimplemented!()
        }
    }

    pub fn into_with_primitives(self) -> TypeResult<Self> {
        let python_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("type_checker")
            .join("resources")
            .join("primitives");
        let (mut py_types, mut py_fields, mut py_functions) = python_files(&python_dir)?;

        let mut types = self.types.clone();
        let mut functions = self.functions.clone();
        let mut fields = self.fields.clone();

        types.append(&mut py_types);
        functions.append(&mut py_functions);
        fields.append(&mut py_fields);

        Ok(Context { types, functions, fields })
    }
}

impl TryFrom<&[CheckInput]> for Context {
    type Error = Vec<TypeErr>;

    fn try_from(files: &[CheckInput]) -> Result<Self, Self::Error> {
        let (types, fields, functions) = generics(files)?;
        Ok(Context { types, functions, fields })
    }
}
