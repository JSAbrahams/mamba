use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

use crate::check::context::arg::FunctionArg;
use crate::check::context::clss::generic::GenericClass;
use crate::check::context::field::Field;
use crate::check::context::function::Function;
use crate::check::context::name::{DirectName, Name, NameUnion};
use crate::check::context::{Context, LookupClass};
use crate::check::result::{TypeErr, TypeResult};
use crate::common::delimit::newline_delimited;
use crate::common::position::Position;

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

pub mod generic;
pub mod python;

/// Concrete type.
///
/// Has fields and functions defined within and from parents for easy access.
///
/// Parents are immediate parents.
///
/// Vector of fields signifies the fields of self, followed by fields of
/// consecutive parents and their parents. The same goes for functions.
#[derive(Debug, Clone, Eq)]
pub struct Class {
    pub is_py_type: bool,
    pub name:       DirectName,
    pub concrete:   bool,
    pub args:       Vec<FunctionArg>,
    pub fields:     HashSet<Field>,
    pub parents:    HashSet<DirectName>,
    pub functions:  HashSet<Function>
}

pub trait HasParent<T> {
    /// Has name as parent.
    ///
    /// Does recursive search. Is true if any ancestor is equal to name.
    fn has_parent(&self, name: T, ctx: &Context, pos: &Position) -> TypeResult<bool>;
}

impl HasParent<&DirectName> for Class {
    fn has_parent(
        &self,
        name: &DirectName,
        ctx: &Context,
        pos: &Position
    ) -> Result<bool, Vec<TypeErr>> {
        Ok(self.parents.contains(name) || {
            let res: Vec<bool> = self
                .parents
                .iter()
                .map(|p| ctx.class(p, pos)?.has_parent(name, ctx, pos))
                .collect::<Result<_, _>>()?;
            res.iter().any(|b| *b)
        })
    }
}

impl HasParent<&NameUnion> for Class {
    fn has_parent(
        &self,
        name: &NameUnion,
        ctx: &Context,
        pos: &Position
    ) -> Result<bool, Vec<TypeErr>> {
        let name = name.as_direct("class", pos)?;
        let res: Vec<bool> =
            name.iter().map(|p| self.has_parent(p, ctx, pos)).collect::<Result<_, _>>()?;
        Ok(res.iter().all(|b| *b))
    }
}

impl Hash for Class {
    fn hash<H: Hasher>(&self, state: &mut H) { self.name.hash(state) }
}

impl PartialEq for Class {
    fn eq(&self, other: &Self) -> bool { self.name == other.name }
}

impl Display for Class {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result { write!(f, "{}", self.name) }
}

impl TryFrom<(&GenericClass, &HashMap<String, Name>, &HashSet<GenericClass>, &Position)> for Class {
    type Error = Vec<TypeErr>;

    fn try_from(
        (generic, generics, types, pos): (
            &GenericClass,
            &HashMap<String, Name>,
            &HashSet<GenericClass>,
            &Position
        )
    ) -> Result<Self, Self::Error> {
        let self_name = generic.name.substitute(generics, pos)?;
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

        let mut parents: HashSet<DirectName> = HashSet::new();
        for parent in &generic.parents {
            let ty = types.iter().find(|ty| ty.name == parent.name).ok_or_else(|| {
                TypeErr::new(pos, &format!("Unknown parent type: {}", parent.name))
            })?;

            let ty = Class::try_from((ty, generics, types, pos))?;
            fields = fields.union(&ty.fields).cloned().collect();
            functions = functions.union(&ty.functions).cloned().collect();
            parents.insert(parent.name.clone());
        }

        Ok(Class {
            is_py_type: generic.is_py_type,
            name: self_name,
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

impl Class {
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

    pub fn function(&self, fun_name: &DirectName, pos: &Position) -> TypeResult<Function> {
        self.functions
            .iter()
            .find_map(
                |function| {
                    if function.name == fun_name.clone() {
                        Some(function.clone())
                    } else {
                        None
                    }
                }
            )
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
