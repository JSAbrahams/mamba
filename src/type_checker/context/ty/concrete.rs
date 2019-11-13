use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

use crate::common::position::Position;
use crate::type_checker::context::field::concrete::Field;
use crate::type_checker::context::function::concrete::Function;
use crate::type_checker::context::function_arg::concrete::{args_compatible, FunctionArg};
use crate::type_checker::context::ty::generic::GenericType;
use crate::type_checker::context::ty::python;
use crate::type_checker::context::type_name::actual::ActualTypeName;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use crate::type_checker::util::{comma_delimited, newline_delimited};

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

pub const NONE: &'static str = "undefined";
pub const EXCEPTION: &'static str = "Exception";

// TODO change

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Type {
    pub is_py_type: bool,
    pub name:       ActualTypeName,
    pub concrete:   bool,
    pub args:       Vec<FunctionArg>,
    fields:         HashSet<Field>,
    functions:      HashSet<Function>
}

impl Hash for Type {
    fn hash<H: Hasher>(&self, state: &mut H) { self.name.hash(state) }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result { write!(f, "{}", self.name) }
}

impl TryFrom<(&GenericType, &HashMap<String, TypeName>, &HashSet<GenericType>, &Position)>
    for Type
{
    type Error = Vec<TypeErr>;

    fn try_from(
        (generic, generics, types, pos): (
            &GenericType,
            &HashMap<String, TypeName>,
            &HashSet<GenericType>,
            &Position
        )
    ) -> Result<Self, Self::Error> {
        let mut fields: HashSet<Field> = generic
            .fields
            .iter()
            .map(|field| Field::try_from((field, generics, pos)))
            .collect::<Result<_, _>>()?;
        let mut functions: HashSet<Function> = generic
            .functions
            .iter()
            .map(|fun| Function::try_from((fun, generics, pos)))
            .collect::<Result<_, _>>()?;

        for parent in &generic.parents {
            let name = TypeName::from(parent.name.as_str()).single(pos)?;
            let ty = types
                .iter()
                .find(|ty| ty.name == name)
                .ok_or(TypeErr::new(pos, &format!("Unknown parent type: {}", name)))?;

            let ty = Type::try_from((ty, generics, types, pos))?;
            fields = fields.union(&ty.fields).cloned().collect();
            functions = functions.union(&ty.functions).cloned().collect();
        }

        Ok(Type {
            is_py_type: generic.is_py_type,
            name: generic.name.substitute(generics, pos)?,
            concrete: generic.concrete,
            args: generic
                .args
                .iter()
                .map(|a| FunctionArg::try_from((a, generics, pos)))
                .collect::<Result<_, _>>()?,
            fields,
            functions
        })
    }
}

impl Type {
    pub fn field(&self, name: &str, pos: &Position) -> TypeResult<Field> {
        let field = self.fields.iter().find(|field| field.name.as_str() == name).cloned();
        field.ok_or(vec![TypeErr::new(
            pos,
            &format!(
                "Type {} does not define field {}, must be one of:\n{}",
                self.name,
                name,
                newline_delimited(&self.fields)
            )
        )])
    }

    // TODO add boolean for unsafe operator so we can ignore if type is None
    // TODO handle default arguments
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
                    if name.as_str() == fun_name && args_compatible(&function.arguments, &args) {
                        Some(Ok(function.clone()))
                    } else {
                        None
                    },
            })
            .ok_or_else(|| {
                // TODO when type inference is more advanced insert expected type
                vec![TypeErr::new(
                    pos,
                    &format!(
                        "Type {} does not have function \"{}: ({}) -> ?\", must be one of: \n{}",
                        self,
                        fun_name,
                        comma_delimited(args),
                        newline_delimited(&self.functions)
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

        NONE => String::from(python::NONE),
        other => String::from(other)
    }
}
