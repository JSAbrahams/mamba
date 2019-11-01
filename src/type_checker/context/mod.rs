use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::ops::Deref;
use std::path::PathBuf;

use crate::common::position::Position;
use crate::type_checker::context::field::generic::GenericField;
use crate::type_checker::context::function::concrete::Function;
use crate::type_checker::context::function::generic::GenericFunction;
use crate::type_checker::context::function_arg::concrete::args_compatible;
use crate::type_checker::context::generics::generics;
use crate::type_checker::context::python::python_files;
use crate::type_checker::context::ty::concrete;
use crate::type_checker::context::ty::concrete::Type;
use crate::type_checker::context::ty::generic::GenericType;
use crate::type_checker::context::type_name::actual::ActualTypeName;
use crate::type_checker::context::type_name::nullable::NullableTypeName;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::environment::expression_type::actual_type::ActualType;
use crate::type_checker::environment::expression_type::nullable_type::NullableType;
use crate::type_checker::environment::expression_type::ExpressionType;
use crate::type_checker::environment::infer_type::InferType;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use crate::type_checker::util::comma_delimited;
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
    types:     HashSet<GenericType>,
    functions: HashSet<GenericFunction>,
    fields:    HashSet<GenericField>
}

impl TryFrom<&[CheckInput]> for Context {
    type Error = Vec<TypeErr>;

    fn try_from(files: &[CheckInput]) -> Result<Self, Self::Error> {
        let (types, fields, functions) = generics(files)?;
        Ok(Context { types, functions, fields })
    }
}

impl Context {
    fn find_type_name(&self, name: &str, pos: &Position) -> TypeResult<GenericType> {
        for ty in &self.types {
            if &ty.name.name(pos)? == name {
                return Ok(ty.clone());
            }
        }
        Err(vec![TypeErr::new(pos, &format!("Unknown type: {}", name))])
    }

    fn lookup_direct(
        &self,
        name: &str,
        generics: &Vec<TypeName>,
        pos: &Position
    ) -> TypeResult<Type> {
        let generic_type: GenericType = self.find_type_name(name, pos)?;
        if generic_type.generics.len() == generics.len() {
            let generics: HashMap<_, _> = generic_type
                .clone()
                .generics
                .into_iter()
                .zip(generics.clone())
                .map(|(parameter, type_name)| (parameter.name, type_name))
                .collect();
            Type::try_from((&generic_type, &generics, pos))
        } else {
            Err(vec![TypeErr::new(
                pos,
                &format!(
                    "Type takes {} generic arguments, but given {}: [{}]",
                    generic_type.generics.len(),
                    generics.len(),
                    comma_delimited(generics)
                )
            )])
        }
    }

    fn lookup_actual(
        &self,
        ty_name: &NullableTypeName,
        pos: &Position
    ) -> TypeResult<NullableType> {
        Ok(NullableType::new(
            ty_name.is_nullable || ty_name.actual == ActualTypeName::new(concrete::NONE, &vec![]),
            &match &ty_name.actual {
                ActualTypeName::Single { lit, generics } =>
                    ActualType::Single { ty: self.lookup_direct(lit, generics, pos)? },
                ActualTypeName::Tuple { ty_names } => ActualType::Tuple {
                    types: ty_names
                        .iter()
                        .map(|ty_name| self.lookup(ty_name, pos))
                        .collect::<Result<_, _>>()?
                },
                ActualTypeName::AnonFun { args, ret_ty } => {
                    let args =
                        args.iter().map(|arg| self.lookup(arg, pos)).collect::<Result<_, _>>()?;
                    ActualType::AnonFun {
                        args,
                        ret_ty: Box::new(self.lookup(ret_ty.deref(), pos)?)
                    }
                }
            }
        ))
    }

    fn lookup_actual_fun(
        &self,
        fun_name: &ActualTypeName,
        fun_args: &[TypeName],
        pos: &Position
    ) -> TypeResult<InferType> {
        // TODO do something with generics
        let (name, ..) = match fun_name {
            ActualTypeName::Single { lit, generics } => (lit.clone(), generics.clone()),
            _ => return Err(vec![TypeErr::new(pos, "Must be type name")])
        };

        let name = ActualTypeName::new(&name, &vec![]);

        // TODO deal with arguments that have no type (in unsafe mode)
        // TODO use generics for arguments
        let fun = self
            .functions
            .iter()
            .find(|f| f.name == name)
            .ok_or(vec![TypeErr::new(pos, &format!("Function {} is undefined", name))])?;

        let fun = Function::try_from((fun, &HashMap::new(), pos))?;
        if !args_compatible(&fun.arguments, &fun_args) {
            return Err(vec![TypeErr::new(pos, "arguments not compatible")]);
        }

        let raises = fun.raises.clone().into_iter().collect();
        if let Some(ret_ty) = &fun.ty() {
            let infer_ty = InferType::from(&self.lookup(&ret_ty, pos)?);
            Ok(infer_ty.union_raises(&raises))
        } else {
            Ok(InferType::new().union_raises(&raises))
        }
    }

    pub fn lookup(&self, type_name: &TypeName, pos: &Position) -> TypeResult<ExpressionType> {
        match type_name {
            TypeName::Single { ty } =>
                Ok(ExpressionType::Single { ty: self.lookup_actual(ty, pos)? }),
            TypeName::Union { union } => {
                let union: HashSet<_> = union
                    .iter()
                    .map(|a_t| self.lookup_actual(a_t, pos))
                    .collect::<Result<_, Vec<TypeErr>>>()?;
                Ok(ExpressionType::Union { union })
            }
        }
    }

    pub fn lookup_fun(
        &self,
        fun_name: &TypeName,
        args: &[TypeName],
        pos: &Position
    ) -> TypeResult<InferType> {
        match fun_name {
            TypeName::Single { ty } => self.lookup_actual_fun(&ty.actual, args, pos),
            TypeName::Union { union } => {
                let union: Vec<InferType> = union
                    .iter()
                    .map(|ty| self.lookup_actual_fun(&ty.actual, args, pos))
                    .collect::<Result<_, Vec<TypeErr>>>()?;

                let mut first = union.first().ok_or(TypeErr::new(pos, "Union is empty"))?.clone();
                for infer_ty in union {
                    first = first.union(&infer_ty, pos)?
                }

                Ok(first)
            }
        }
    }

    /// Loads pre-defined Python primitives into context for easy lookup
    pub fn into_with_primitives(self) -> TypeResult<Self> {
        let python_dir = resource("primitives");
        let (py_types, py_fields, py_functions) = python_files(&python_dir)?;

        let types = self.types.union(&py_types).cloned().collect();
        let functions = self.functions.union(&py_functions).cloned().collect();
        let fields = self.fields.union(&py_fields).cloned().collect();
        Ok(Context { types, functions, fields })
    }

    pub fn into_with_std_lib(self) -> TypeResult<Self> {
        let python_dir = resource("std_lib");
        let (py_types, py_fields, py_functions) = python_files(&python_dir)?;

        let types = self.types.union(&py_types).cloned().collect();
        let functions = self.functions.union(&py_functions).cloned().collect();
        let fields = self.fields.union(&py_fields).cloned().collect();
        Ok(Context { types, functions, fields })
    }
}

fn resource(resource: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("type_checker")
        .join("resources")
        .join(resource)
}
