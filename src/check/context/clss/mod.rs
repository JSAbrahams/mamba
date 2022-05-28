use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

use crate::check::context::{Context, LookupClass};
use crate::check::context::arg::FunctionArg;
use crate::check::context::arg::generic::GenericFunctionArg;
use crate::check::context::clss::generic::GenericClass;
use crate::check::context::field::Field;
use crate::check::context::field::generic::GenericField;
use crate::check::context::function::Function;
use crate::check::context::function::generic::GenericFunction;
use crate::check::name::Name;
use crate::check::name::stringname::StringName;
use crate::check::name::truename::TrueName;
use crate::check::result::{TypeErr, TypeResult};
use crate::common::position::Position;

pub const INT_PRIMITIVE: &str = "Int";
pub const FLOAT_PRIMITIVE: &str = "Float";
pub const STRING_PRIMITIVE: &str = "String";
pub const BOOL_PRIMITIVE: &str = "Bool";
pub const ENUM_PRIMITIVE: &str = "Enum";
pub const COMPLEX_PRIMITIVE: &str = "Complex";

pub const COLLECTION: &str = "Collection";
pub const RANGE: &str = "Range";
pub const SLICE: &str = "Slice";
pub const SET: &str = "Set";
pub const LIST: &str = "List";

pub const NONE: &str = "None";
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
    pub name: TrueName,
    pub concrete: bool,
    pub args: Vec<FunctionArg>,
    pub fields: HashSet<Field>,
    pub parents: HashSet<TrueName>,
    pub functions: HashSet<Function>,
}

pub trait HasParent<T> {
    /// Has name as parent.
    ///
    /// Does recursive search. Is true if any ancestor is equal to name.
    fn has_parent(&self, name: T, ctx: &Context, pos: Position) -> TypeResult<bool>;
}

impl HasParent<&StringName> for Class {
    fn has_parent(&self, name: &StringName, ctx: &Context, pos: Position) -> TypeResult<bool> {
        Ok(&self.name == name
            || self
            .parents
            .iter()
            .map(|p| ctx.class(p, pos)?.has_parent(name, ctx, pos))
            .collect::<Result<Vec<bool>, _>>()?
            .iter()
            .any(|b| *b))
    }
}

impl HasParent<&TrueName> for Class {
    fn has_parent(&self, name: &TrueName, ctx: &Context, pos: Position) -> TypeResult<bool> {
        Ok(&self.name == name
            || self
            .parents
            .iter()
            .map(|p| ctx.class(p, pos)?.has_parent(name, ctx, pos).map(|res| res || p == name))
            .collect::<Result<Vec<bool>, _>>()?
            .iter()
            .any(|b| *b))
    }
}

impl HasParent<&Name> for Class {
    fn has_parent(&self, name: &Name, ctx: &Context, pos: Position) -> TypeResult<bool> {
        if name.contains(&self.name) {
            return Ok(true);
        }

        let names = name.as_direct("Name must be string", pos)?;

        let parent_names: Vec<StringName> = self
            .parents
            .iter()
            .map(|true_name| true_name.as_direct("Has parent", pos))
            .collect::<Result<_, _>>()?;

        let parent_classes: Vec<Class> =
            parent_names.iter().map(|name| ctx.class(name, pos)).collect::<Result<_, _>>()?;

        for name in names {
            let res = parent_classes
                .iter()
                .map(|pc| pc.has_parent(&name, ctx, pos))
                .collect::<Result<Vec<bool>, _>>()?;
            if res.iter().any(|a| *a) {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

impl Hash for Class {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

impl PartialEq for Class {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl TryFrom<(&GenericClass, &HashMap<Name, Name>, Position)> for Class {
    type Error = Vec<TypeErr>;

    fn try_from(
        (generic, generics, pos): (&GenericClass, &HashMap<Name, Name>, Position),
    ) -> Result<Self, Self::Error> {
        let try_arg = |a: &GenericFunctionArg| FunctionArg::try_from((a, generics, pos));
        let try_field = |field: &GenericField| Field::try_from((field, generics, pos));
        let try_function = |fun: &GenericFunction| Function::try_from((fun, generics, pos));

        Ok(Class {
            is_py_type: generic.is_py_type,
            name: generic.name.substitute(generics, pos)?,
            concrete: generic.concrete,
            args: generic.args.iter().map(try_arg).collect::<Result<_, _>>()?,
            parents: generic.parents.iter().map(|g| g.name.clone()).collect(),
            fields: generic.fields.iter().map(try_field).collect::<Result<_, _>>()?,
            functions: generic.functions.iter().map(try_function).collect::<Result<_, _>>()?,
        })
    }
}

impl Class {
    pub fn constructor(&self, without_self: bool, pos: Position) -> TypeResult<Function> {
        Ok(Function {
            is_py_type: false,
            name: self.name.as_direct("function name", pos)?,
            self_mutable: None,
            pure: false,
            arguments: if without_self && !self.args.is_empty() {
                self.args.iter().skip(1).cloned().collect()
            } else {
                self.args.clone()
            },
            raises: Name::empty(),
            in_class: None,
            ret_ty: Name::from(&self.name),
        })
    }

    pub fn field(&self, name: &str, ctx: &Context, pos: Position) -> TypeResult<Field> {
        if let Some(field) = self.fields.iter().find(|f| f.name == name) {
            return Ok(field.clone());
        }

        for parent in &self.parents {
            if let Ok(field) = ctx.class(parent, pos)?.field(name, ctx, pos) {
                return Ok(field);
            }
        }

        Err(vec![TypeErr::new(pos, &format!("'{}' does not define '{}'", self, name))])
    }

    /// Get function of class.
    ///
    /// If class does not implement function, traverse parents until function
    /// found.
    pub fn fun(&self, name: &StringName, ctx: &Context, pos: Position) -> TypeResult<Function> {
        if let Some(function) = self.functions.iter().find(|f| &f.name == name) {
            return Ok(function.clone());
        }

        // TODO deal with conflicting function names in parents.
        // TODO check for cyclic dependencies after constructing Context.
        for parent in &self.parents {
            if let Ok(function) = ctx.class(parent, pos)?.fun(name, ctx, pos) {
                return Ok(function);
            }
        }

        Err(vec![TypeErr::new(pos, &format!("'{}' does not define '{}'", self, name))])
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

        COLLECTION => String::from(python::COLLECTION),
        RANGE => String::from(python::RANGE),
        SLICE => String::from(python::SLICE),
        SET => String::from(python::SET),
        LIST => String::from(python::LIST),

        NONE => String::from(python::NONE),
        EXCEPTION => String::from(python::EXCEPTION),
        other => String::from(other),
    }
}
