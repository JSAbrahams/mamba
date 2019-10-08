use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

use crate::common::position::Position;
use crate::type_checker::context::field::concrete::Field;
use crate::type_checker::context::function::concrete::Function;
use crate::type_checker::context::function_arg::concrete::FunctionArg;
use crate::type_checker::context::ty::generic::GenericType;
use crate::type_checker::context::type_name::actual::ActualTypeName;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::type_result::{TypeErr, TypeResult};

pub const INT_PRIMITIVE: &'static str = "Int";
pub const FLOAT_PRIMITIVE: &'static str = "Float";
pub const STRING_PRIMITIVE: &'static str = "String";
pub const BOOL_PRIMITIVE: &'static str = "Bool";
pub const ENUM_PRIMITIVE: &'static str = "Enum";

pub const RANGE: &'static str = "Range";

// TODO add parents

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Type {
    pub is_py_type: bool,
    pub name:       ActualTypeName,
    pub concrete:   bool,
    pub args:       Vec<FunctionArg>,
    fields:         Vec<Field>,
    functions:      Vec<Function>
}

impl Hash for Type {
    fn hash<H: Hasher>(&self, state: &mut H) { self.name.hash(state) }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result { write!(f, "{}", self.name) }
}

impl TryFrom<(&GenericType, &HashMap<String, ActualTypeName>, &Position)> for Type {
    type Error = Vec<TypeErr>;

    fn try_from(
        (generic, generics, pos): (&GenericType, &HashMap<String, ActualTypeName>, &Position)
    ) -> Result<Self, Self::Error> {
        Ok(Type {
            is_py_type: generic.is_py_type,
            name:       generic.name.substitute(generics, pos)?,
            concrete:   generic.concrete,
            args:       generic
                .args
                .iter()
                .map(|a| FunctionArg::try_from((a, generics, pos)))
                .collect::<Result<_, _>>()?,
            fields:     generic
                .fields
                .iter()
                .map(|f| Field::try_from((f, generics, pos)))
                .collect::<Result<_, _>>()?,
            functions:  generic
                .functions
                .iter()
                .map(|f| Function::try_from((f, generics, pos)))
                .collect::<Result<_, _>>()?
        })
    }
}

impl Type {
    pub fn field(&self, name: &str) -> Option<Field> {
        self.fields.iter().find(|field| field.name.as_str() == name).cloned()
    }

    // TODO add boolean for unsafe operator so we can ignore if type is None
    pub fn fun(
        &self,
        fun_name: &str,
        args: &[TypeName],
        safe: bool,
        pos: &Position
    ) -> TypeResult<Function> {
        // TODO accept if arguments passed is union that is subset of argument union
        self.functions
            .iter()
            .find_map(|function| match function.name.name(pos) {
                Err(err) => Some(Err(err)),
                Ok(name) =>
                    if name.as_str() == fun_name
                        && function
                            .arguments
                            .iter()
                            .map(|arg| arg.ty.clone())
                            .zip(args)
                            .all(|(left, right)| left == Some(right.clone()))
                    {
                        Some(Ok(function.clone()))
                    } else {
                        None
                    },
            })
            .ok_or_else(|| vec![TypeErr::new(pos, "Unknown function")])?
    }
}
