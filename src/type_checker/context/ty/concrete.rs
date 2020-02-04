use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

use crate::common::position::Position;
use crate::type_checker::checker_result::{TypeErr, TypeResult};
use crate::type_checker::context::field::concrete::Field;
use crate::type_checker::context::function::concrete::Function;
use crate::type_checker::context::function_arg::concrete::{args_compatible, FunctionArg};
use crate::type_checker::context::ty::generic::GenericType;
use crate::type_checker::context::ty::python;
use crate::type_checker::ty_name::actual::ActualTypeName;
use crate::type_checker::ty_name::TypeName;
use crate::type_checker::util::{comma_delimited, newline_delimited};

pub const ANY: &str = "Any";

pub const INT_PRIMITIVE: &str = "Int";
pub const FLOAT_PRIMITIVE: &str = "Float";
pub const STRING_PRIMITIVE: &str = "String";
pub const BOOL_PRIMITIVE: &str = "Bool";
pub const ENUM_PRIMITIVE: &str = "Enum";
pub const COMPLEX_PRIMITIVE: &str = "Complex";

pub const RANGE: &str = "Range";
pub const SET: &str = "Set";
pub const LIST: &str = "List";

pub const NONE: &str = "undefined";
pub const EXCEPTION: &str = "Exception";

/// Concrete type.
///
/// Has fields and functions defined within and from parents for easy access.
///
/// Parents are immediate parents.
#[derive(Debug, Clone, Eq)]
pub struct Type {
    pub is_py_type: bool,
    pub name:       ActualTypeName,
    pub concrete:   bool,
    pub args:       Vec<FunctionArg>,
    pub fields:     HashSet<Field>,
    pub parents:    HashSet<TypeName>,
    functions:      HashSet<Function>
}

impl Hash for Type {
    fn hash<H: Hasher>(&self, state: &mut H) { self.name.hash(state) }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool { self.name == other.name }
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

        let mut parents: HashSet<TypeName> = HashSet::new();
        for parent in &generic.parents {
            let name = TypeName::from(parent.name.as_str()).single(pos)?;
            let ty = types
                .iter()
                .find(|ty| ty.name == name)
                .ok_or_else(|| TypeErr::new(pos, &format!("Unknown parent type: {}", name)))?;

            let ty = Type::try_from((ty, generics, types, pos))?;
            fields = fields.union(&ty.fields).cloned().collect();
            functions = functions.union(&ty.functions).cloned().collect();
            parents.insert(TypeName::from(&name));
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
            parents,
            fields,
            functions
        })
    }
}

impl Type {
    pub fn field(&self, name: &str, pos: &Position) -> TypeResult<Field> {
        let field = self.fields.iter().find(|field| field.name.as_str() == name).cloned();
        field.ok_or_else(|| {
            vec![TypeErr::new(
                pos,
                &format!(
                    "Type {} does not define field \"{}\"{}{}",
                    self.name,
                    name,
                    if self.fields.is_empty() { "" } else { ", must be one of:\n" },
                    newline_delimited(&self.fields)
                )
            )]
        })
    }

    pub fn fun_ret_ty(&self, fun_name: &TypeName, pos: &Position) -> TypeResult<Option<TypeName>> {
        self.functions
            .iter()
            .find_map(|function| {
                if TypeName::from(&function.name) == fun_name.clone() {
                    Some(function.ty())
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                vec![TypeErr::new(
                    pos,
                    &format!(
                        "Type {} does not define function \"{}\"{}{}",
                        self,
                        fun_name,
                        if self.functions.is_empty() { "" } else { ", must be one of:\n" },
                        newline_delimited(&self.functions)
                    )
                )]
            })
    }

    pub fn function(&self, fun_name: &TypeName, pos: &Position) -> TypeResult<Function> {
        self.functions
            .iter()
            .find_map(|function| {
                if TypeName::from(&function.name) == fun_name.clone() {
                    Some(function.clone())
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                vec![TypeErr::new(
                    pos,
                    &format!(
                        "Type {} does not define function \"{}\"{}{}",
                        self,
                        fun_name,
                        if self.functions.is_empty() { "" } else { ", must be one of: \n" },
                        newline_delimited(&self.functions)
                    )
                )]
            })
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

pub fn concrete_to_python(name: &str) -> String {
    match name {
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
