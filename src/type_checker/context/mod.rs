use std::convert::TryFrom;
use std::path::PathBuf;

use crate::common::position::Position;
use crate::type_checker::context::concrete::Type;
use crate::type_checker::context::generic::field::GenericField;
use crate::type_checker::context::generic::function::GenericFunction;
use crate::type_checker::context::generic::generics;
use crate::type_checker::context::generic::ty::GenericType;
use crate::type_checker::context::generic::type_name::GenericTypeName;
use crate::type_checker::context::python::python_files;
use crate::type_checker::environment::actual_type::ActualType;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use crate::type_checker::CheckInput;

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
    types:     Vec<GenericType>,
    functions: Vec<GenericFunction>,
    fields:    Vec<GenericField>
}

impl TryFrom<&[CheckInput]> for Context {
    type Error = Vec<TypeErr>;

    fn try_from(files: &[CheckInput]) -> Result<Self, Self::Error> {
        let (types, fields, functions) = generics(files)?;
        Ok(Context { types, functions, fields })
    }
}

impl Context {
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

        if generic_type.generics.len() == generics.len() {
            let generics = generic_type
                .generics
                .clone()
                .iter()
                .zip(generics)
                .map(|(parameter, type_name)| (parameter.name.clone(), type_name.clone()))
                .collect();

            Type::try_from(generic_type, &generics, pos)
        } else {
            Err(TypeErr::new(
                pos,
                format!("Type takes {} generic arguments", generic_type.generics.len()).as_str()
            ))
        }
    }

    pub fn lookup(
        &self,
        type_name: &GenericTypeName,
        pos: &Position
    ) -> Result<ActualType, TypeErr> {
        Ok(match type_name {
            GenericTypeName::Single { lit, generics } => ActualType::Single {
                expr_ty: ExpressionType::from(self.lookup_direct(lit, generics, pos)?)
            },
            GenericTypeName::Fun { args, ret_ty } => ActualType::Fun {
                args:   args.iter().map(|a| self.lookup(a, pos)).collect::<Result<_, _>>()?,
                ret_ty: Box::from(self.lookup(ret_ty, pos)?)
            },
            GenericTypeName::Union { ty_names } => ActualType::Union {
                types: ty_names.iter().map(|t| self.lookup(t, pos)).collect::<Result<_, _>>()?
            },
            GenericTypeName::Tuple { ty_names } => ActualType::Tuple {
                types: ty_names.iter().map(|t| self.lookup(t, pos)).collect::<Result<_, _>>()?
            }
        })
    }

    /// Loads pre-defined Python primtives into context for easy lookup
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
