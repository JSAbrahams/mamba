use std::convert::TryFrom;
use std::path::PathBuf;

use crate::common::position::Position;
use crate::type_checker::context::concrete::type_name::actual::ActualTypeName;
use crate::type_checker::context::concrete::type_name::TypeName;
use crate::type_checker::context::concrete::Type;
use crate::type_checker::context::generic::field::GenericField;
use crate::type_checker::context::generic::function::GenericFunction;
use crate::type_checker::context::generic::generics;
use crate::type_checker::context::generic::ty::GenericType;
use crate::type_checker::context::python::python_files;
use crate::type_checker::environment::expression_type::actual_type::ActualType;
use crate::type_checker::environment::expression_type::mutable_type::MutableType;
use crate::type_checker::environment::expression_type::ExpressionType;
use crate::type_checker::environment::infer_type::InferType;
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
        generics: &[ActualTypeName],
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

    fn lookup_actual(&self, ty_name: &ActualTypeName, pos: &Position) -> TypeResult<MutableType> {
        let ty = match ty_name {
            ActualTypeName::Single { lit, generics } => self.lookup_direct(lit, generics, pos),
            ActualTypeName::Tuple { ty_names } => unimplemented!(),
            ActualTypeName::AnonFun { args, ret_ty } => unimplemented!()
        }?;

        Ok(MutableType::from(&ActualType::from(&ty)))
    }

    pub fn lookup(&self, type_name: &TypeName, pos: &Position) -> TypeResult<InferType> {
        let expr_type = match type_name {
            TypeName::Single { ty } =>
                ExpressionType::Single { mut_ty: self.lookup_actual(ty, pos)? },
            TypeName::Union { union } => ExpressionType::Union {
                union: union.iter().map(|a_t| self.lookup_actual(a_t, pos)).collect()?
            }
        };
        Ok(InferType { raises: vec![], expr_type: Some(expr_type) })
    }

    pub fn lookup_fun(&self, name: &str, pos: &Position) {}

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
