use std::collections::HashMap;

use crate::common::position::Position;
use crate::type_checker::context::concrete::field::Field;
use crate::type_checker::context::concrete::function::Function;
use crate::type_checker::context::concrete::function_arg::FunctionArg;
use crate::type_checker::context::concrete::type_name::TypeName;
use crate::type_checker::context::generic::type_name::GenericTypeName;
use crate::type_checker::context::generic::GenericType;
use crate::type_checker::type_result::TypeErr;
use std::fmt;
use std::fmt::{Display, Formatter};

pub mod field;
pub mod function;
pub mod function_arg;
pub mod type_name;

// TODO add parents

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Type {
    pub is_py_type: bool,
    pub name:       TypeName,
    pub concrete:   bool,
    pub args:       Vec<FunctionArg>,
    pub fields:     Vec<Field>,
    pub functions:  Vec<Function>
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result { write!(f, "{}", self.name) }
}

impl Type {
    pub fn try_from(
        generic_type: &GenericType,
        generics: &HashMap<String, GenericTypeName>,
        pos: &Position
    ) -> Result<Self, TypeErr> {
        Ok(Type {
            is_py_type: generic_type.is_py_type,
            name:       TypeName::Single {
                lit:      generic_type.name.clone(),
                generics: generic_type
                    .generics
                    .iter()
                    .map(|g| TypeName::Single { lit: g.name.clone(), generics: vec![] })
                    .collect()
            },
            concrete:   generic_type.concrete,
            args:       generic_type
                .args
                .iter()
                .map(|a| FunctionArg::try_from(a, generics, pos))
                .collect::<Result<_, _>>()?,
            fields:     generic_type
                .fields
                .iter()
                .map(|f| Field::try_from(f, generics, pos))
                .collect::<Result<_, _>>()?,
            functions:  generic_type
                .functions
                .iter()
                .map(|f| Function::try_from(f, generics, pos))
                .collect::<Result<_, _>>()?
        })
    }

    pub fn overrides_function(&self, fun_name: &str) -> bool {
        self.functions.iter().any(|function| function.name.as_str() == fun_name)
    }
}
