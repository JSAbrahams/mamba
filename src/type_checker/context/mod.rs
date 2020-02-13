use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::ops::Deref;
use std::path::PathBuf;

use crate::common::position::Position;
use crate::type_checker::checker_result::{TypeErr, TypeResult};
use crate::type_checker::context::field::generic::GenericField;
use crate::type_checker::context::function::concrete::Function;
use crate::type_checker::context::function::generic::GenericFunction;
use crate::type_checker::context::generics::generics;
use crate::type_checker::context::python::python_files;
use crate::type_checker::context::ty::concrete;
use crate::type_checker::context::ty::concrete::Type;
use crate::type_checker::context::ty::generic::GenericType;
use crate::type_checker::ty::actual::ActualType;
use crate::type_checker::ty::nullable::NullableType;
use crate::type_checker::ty::ExpressionType;
use crate::type_checker::ty_name::actual::ActualTypeName;
use crate::type_checker::ty_name::nullable::NullableTypeName;
use crate::type_checker::ty_name::TypeName;
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
    pub fn lookup_name(&self, name: &TypeName, pos: &Position) -> TypeResult<TypeName> {
        let expr_type = self.lookup(name, pos)?;
        Ok(TypeName::from(&expr_type))
    }

    fn find_type_name(&self, name: &str, pos: &Position) -> TypeResult<GenericType> {
        for ty in &self.types {
            if ty.name.name(pos)? == name {
                return Ok(ty.clone());
            }
        }
        Err(vec![TypeErr::new(pos, &format!("Unknown type: {}", name))])
    }

    fn lookup_direct(&self, name: &str, generics: &[TypeName], pos: &Position) -> TypeResult<Type> {
        let generic_type: GenericType = self.find_type_name(name, pos)?;
        if generic_type.generics.len() != generics.len() {
            let msg = format!(
                "{} takes {} generic arguments: [{}],\nbut given {}: [{}]",
                name,
                generic_type.generics.len(),
                comma_delimited(&generic_type.generics),
                generics.len(),
                comma_delimited(generics)
            );
            return Err(vec![TypeErr::new(pos, &msg)]);
        }

        let type_generics = generic_type.generics.clone();
        let paired = type_generics.into_iter().zip(generics);
        let generics =
            paired.map(|(parameter, type_name)| (parameter.name, type_name.clone())).collect();
        Type::try_from((&generic_type, &generics, &self.types, pos))
    }

    fn lookup_actual(
        &self,
        ty_name: &NullableTypeName,
        pos: &Position
    ) -> TypeResult<NullableType> {
        Ok(NullableType::new(
            ty_name.is_nullable || ty_name.actual == ActualTypeName::new(concrete::NONE, &[]),
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

    fn lookup_actual_function(
        &self,
        name: &ActualTypeName,
        pos: &Position
    ) -> TypeResult<Function> {
        let fun =
            self.functions.iter().find(|f| f.name == name.clone()).ok_or_else(|| {
                vec![TypeErr::new(pos, &format!("Function {} is undefined", name))]
            })?;
        Function::try_from((fun, &HashMap::new(), pos))
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

    pub fn lookup_function(
        &self,
        name: &TypeName,
        pos: &Position
    ) -> TypeResult<HashSet<Function>> {
        match name {
            TypeName::Single { ty } => {
                let mut function_arg_set: HashSet<Function> = HashSet::new();
                function_arg_set.insert(self.lookup_actual_function(&ty.actual, pos)?);
                Ok(function_arg_set)
            }
            TypeName::Union { union } => union
                .iter()
                .map(|ty| self.lookup_actual_function(&ty.actual, pos))
                .collect::<Result<_, Vec<TypeErr>>>()
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

pub fn check_if_parent(
    field: &TypeName,
    in_class: &Vec<TypeName>,
    object_class: &TypeName,
    ctx: &Context,
    pos: &Position
) -> TypeResult<()> {
    let mut in_a_parent = false;
    for class in in_class {
        let is_parent = ctx.lookup(&class, pos)?.has_parent(object_class, ctx, pos)?;
        in_a_parent = in_a_parent || is_parent;
        if in_a_parent {
            break;
        }
    }

    if in_a_parent {
        Ok(())
    } else {
        let msg = if let Some(class) = in_class.last() {
            format!("Cannot access private {} of a {} while in a {}", field, object_class, class)
        } else {
            format!("Cannot access private {} of a {}", field, object_class)
        };
        Err(vec![TypeErr::new(pos, &msg)])
    }
}
