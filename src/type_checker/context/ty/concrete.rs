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
use crate::type_checker::context::ty::python;
use crate::type_checker::context::type_name::actual::ActualTypeName;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::type_result::{TypeErr, TypeResult};

pub const ANY: &'static str = "Any";

pub const INT_PRIMITIVE: &'static str = "Int";
pub const FLOAT_PRIMITIVE: &'static str = "Float";
pub const STRING_PRIMITIVE: &'static str = "String";
pub const BOOL_PRIMITIVE: &'static str = "Bool";
pub const ENUM_PRIMITIVE: &'static str = "Enum";
pub const COMPLEX_PRIMITIVE: &'static str = "Complex";

pub const RANGE: &'static str = "Range";
pub const SET: &'static str = "Set";
pub const LIST: &'static str = "List";

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

impl TryFrom<(&GenericType, &HashMap<String, TypeName>, &Position)> for Type {
    type Error = Vec<TypeErr>;

    fn try_from(
        (generic, generics, pos): (&GenericType, &HashMap<String, TypeName>, &Position)
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
    pub fn fun(&self, fun_name: &str, args: &[TypeName], pos: &Position) -> TypeResult<Function> {
        let args: Vec<TypeName> = vec![vec![TypeName::from(&self.name.clone())], args.to_vec()]
            .into_iter()
            .flatten()
            .collect();
        // TODO accept if arguments passed is union that is subset of argument union
        self.functions
            .iter()
            .find_map(|function| match function.name.name(pos) {
                Err(err) => Some(Err(err)),
                Ok(name) =>
                    if name.as_str() == fun_name
                        && function.arguments.len() == args.len()
                        && function
                            .arguments
                            .iter()
                            .map(|arg| arg.ty.clone())
                            .zip(args.clone())
                            .all(|(left, right)| left == Some(right.clone()))
                    {
                        Some(Ok(function.clone()))
                    } else {
                        None
                    },
            })
            .ok_or_else(|| {
                vec![TypeErr::new(
                    pos,
                    &format!(
                        "Type {} does not have function \"{}\": ({}) -> ?, must be one of: {}",
                        self,
                        fun_name,
                        {
                            let mut string = String::new();
                            args.iter().for_each(|fun| string.push_str(&format!("{}, ", fun)));
                            if string.len() > 2 {
                                string.remove(string.len() - 2);
                            }
                            string
                        }
                        .trim_end(),
                        {
                            let mut string = String::new();
                            self.functions
                                .iter()
                                .for_each(|fun| string.push_str(&format!("{}\n", fun)));
                            string
                        }
                    )
                )]
            })?
    }
}

pub fn concrete_to_python(name: &String) -> String {
    match name.as_str() {
        INT_PRIMITIVE => String::from(python::INT_PRIMITIVE),
        FLOAT_PRIMITIVE => String::from(python::FLOAT_PRIMITIVE),
        STRING_PRIMITIVE => String::from(python::STRING_PRIMITIVE),
        BOOL_PRIMITIVE => String::from(python::BOOL_PRIMITIVE),
        ENUM_PRIMITIVE => String::from(python::ENUM_PRIMITIVE),
        COMPLEX_PRIMITIVE => String::from(python::COMPLEX_PRIMITIVE),

        RANGE => String::from(python::RANGE),
        SET => String::from(python::SET),
        LIST => String::from(python::LIST),

        other => String::from(other)
    }
}
