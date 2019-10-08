use std::collections::HashSet;
use std::convert::TryFrom;
use std::path::PathBuf;

use crate::common::position::Position;
use crate::type_checker::context::field::generic::GenericField;
use crate::type_checker::context::function::generic::GenericFunction;
use crate::type_checker::context::generics::generics;
use crate::type_checker::context::python::python_files;
use crate::type_checker::context::ty::concrete::Type;
use crate::type_checker::context::ty::generic::GenericType;
use crate::type_checker::context::type_name::actual::ActualTypeName;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::environment::expression_type::actual_type::ActualType;
use crate::type_checker::environment::expression_type::mutable_type::MutableType;
use crate::type_checker::environment::expression_type::ExpressionType;
use crate::type_checker::environment::infer_type::InferType;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use crate::type_checker::CheckInput;

pub mod field;
pub mod function;
pub mod function_arg;
pub mod parameter;
pub mod parent;
pub mod python;
pub mod ty;
pub mod type_name;

mod generics;

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
    // TODO change so it accepts all actual type name variants
    fn lookup_actual(&self, ty_name: &ActualTypeName, pos: &Position) -> TypeResult<MutableType> {
        let (name, generics) = match ty_name {
            ActualTypeName::Single { lit, generics } => (lit.clone(), generics.clone()),
            _ => return Err(vec![TypeErr::new(pos, "Only can look up using single type")])
        };

        let generic_type: GenericType = self
            .types
            .iter()
            .find_map(|ty| match ty.name.name(pos) {
                Ok(ty_name) =>
                    if ty_name == name {
                        Some(Ok(ty.clone()))
                    } else {
                        None
                    },
                Err(err) => Some(Err(err))
            })
            .ok_or_else(|| vec![TypeErr::new(pos, "Unknown type")])??;
        if generic_type.generics.len() == generics.len() {
            let generics = generic_type
                .clone()
                .generics
                .into_iter()
                .zip(generics.clone())
                .map(|(parameter, type_name)| (parameter.name, type_name))
                .collect();
            let ty = Type::try_from((&generic_type, &generics, pos))?;
            Ok(MutableType::from(&ActualType::from(&ty)))
        } else {
            Err(vec![TypeErr::new(
                pos,
                format!("Type takes {} generic arguments", generic_type.generics.len()).as_str()
            )])
        }
    }

    fn lookup_actual_fun(
        &self,
        fun_name: &ActualTypeName,
        fun_args: &[TypeName],
        pos: &Position
    ) -> TypeResult<MutableType> {
        let (name, generics) = match fun_name {
            ActualTypeName::Single { lit, generics } => (lit.clone(), generics.clone()),
            _ => return Err(vec![TypeErr::new(pos, "Must be type name")])
        };

        unimplemented!()
    }

    pub fn lookup(&self, type_name: &TypeName, pos: &Position) -> TypeResult<InferType> {
        let expr_type = match type_name {
            TypeName::Single { ty } =>
                ExpressionType::Single { mut_ty: self.lookup_actual(ty, pos)? },
            TypeName::Union { union } => {
                let union: HashSet<_> = union
                    .iter()
                    .map(|a_t| self.lookup_actual(a_t, pos))
                    .collect::<Result<_, Vec<TypeErr>>>()?;
                ExpressionType::Union { union }
            }
        };

        Ok(InferType::from(&expr_type))
    }

    pub fn lookup_fun(
        &self,
        fun_name: &TypeName,
        args: &[TypeName],
        pos: &Position
    ) -> TypeResult<InferType> {
        let expr_ty = match fun_name {
            TypeName::Single { ty } =>
                ExpressionType::Single { mut_ty: self.lookup_actual_fun(ty, args, pos)? },
            TypeName::Union { union } => {
                let union: HashSet<_> = union
                    .iter()
                    .map(|a_t| self.lookup_actual_fun(a_t, args, pos))
                    .collect::<Result<_, Vec<TypeErr>>>()?;

                ExpressionType::Union { union }
            }
        };

        Ok(InferType::from(&expr_ty))
    }

    /// Loads pre-defined Python primitives into context for easy lookup
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

    pub fn into_with_std_lib(self) -> TypeResult<Self> {
        let python_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("type_checker")
            .join("resources")
            .join("std_lib");
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
