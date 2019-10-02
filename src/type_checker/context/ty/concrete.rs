use crate::common::position::Position;
use crate::type_checker::context::field::concrete::Field;
use crate::type_checker::context::function::concrete::Function;
use crate::type_checker::context::function_arg::concrete::FunctionArg;
use crate::type_checker::context::ty::generic::GenericType;
use crate::type_checker::context::type_name::concrete::TypeName;
use crate::type_checker::context::type_name::generic::GenericTypeName;
use crate::type_checker::type_result::TypeErr;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};

pub const INT_PRIMITIVE: &'static str = "Int";
pub const FLOAT_PRIMITIVE: &'static str = "Float";
pub const STRING_PRIMITIVE: &'static str = "String";
pub const BOOL_PRIMITIVE: &'static str = "Bool";
pub const ENUM_PRIMITIVE: &'static str = "Enum";

// TODO add parents

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Type {
    pub is_py_type: bool,
    pub name:       TypeName,
    pub concrete:   bool,
    pub args:       Vec<FunctionArg>,
    fields:         Vec<Field>,
    functions:      Vec<Function>
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
            name:       TypeName::new(
                &generic_type.name,
                generic_type.generics.iter().map(|g| TypeName::new(&g.name, vec![])).collect()?
            ),
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

    pub fn field(&self, name: &str) -> Option<Field> {
        self.fields.iter().find(|field| field.name.as_str() == name).cloned()
    }

    pub fn function(&self, fun_name: &str, args: &[TypeName]) -> Option<Function> {
        // TODO also accept if arguments passed is union that is subset of argument
        // union
        self.functions
            .iter()
            .find(|function| {
                function.name.as_str() == fun_name
                    && function
                        .arguments
                        .iter()
                        .map(|arg| arg.ty.clone())
                        .zip(args)
                        .all(|(left, right)| left == Some(right.clone()))
            })
            .cloned()
    }
}
