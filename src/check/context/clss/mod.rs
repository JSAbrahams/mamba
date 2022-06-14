use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

use itertools::{EitherOrBoth, enumerate, Itertools};

use crate::check::context::{clss, Context, LookupClass};
use crate::check::context::arg::FunctionArg;
use crate::check::context::arg::generic::GenericFunctionArg;
use crate::check::context::clss::generic::GenericClass;
use crate::check::context::field::Field;
use crate::check::context::field::generic::GenericField;
use crate::check::context::function::Function;
use crate::check::context::function::generic::GenericFunction;
use crate::check::name::{Empty, Name, Substitute};
use crate::check::name::string_name::StringName;
use crate::check::name::true_name::TrueName;
use crate::check::result::{TypeErr, TypeResult};
use crate::common::position::Position;

pub const INT: &str = "Int";
pub const FLOAT: &str = "Float";
pub const STRING: &str = "String";
pub const BOOL: &str = "Bool";
pub const ENUM: &str = "Enum";
pub const COMPLEX: &str = "Complex";

pub const COLLECTION: &str = "Collection";
pub const RANGE: &str = "Range";
pub const SLICE: &str = "Slice";
pub const SET: &str = "Set";
pub const LIST: &str = "List";
pub const TUPLE: &str = "Tuple";

pub const CALLABLE: &str = "Callable";

pub const NONE: &str = "None";
pub const EXCEPTION: &str = "Exception";

pub mod generic;
pub mod python;
pub mod union;

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
    pub name: StringName,
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

pub trait GetField<T> {
    fn field(&self, name: &str, ctx: &Context, pos: Position) -> TypeResult<T>;
}

pub trait GetFun<T> {
    fn fun(&self, name: &StringName, ctx: &Context, pos: Position) -> TypeResult<T>;
}

impl HasParent<&StringName> for Class {
    fn has_parent(&self, name: &StringName, ctx: &Context, pos: Position) -> TypeResult<bool> {
        Ok(self.name.name == clss::COLLECTION && self.name.name == name.name
            || &self.name == name
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
        self.has_parent(&StringName::from(name), ctx, pos)
    }
}

impl HasParent<&Name> for Class {
    fn has_parent(&self, name: &Name, ctx: &Context, pos: Position) -> TypeResult<bool> {
        if name.contains(&TrueName::from(&self.name)) {
            return Ok(true);
        }

        let names = name.as_direct();
        let parent_names: Vec<StringName> = self.parents.iter().map(StringName::from).collect();
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

impl LookupClass<&TrueName, Class> for Context {
    fn class(&self, class: &TrueName, pos: Position) -> TypeResult<Class> {
        self.class(&StringName::from(class), pos)
    }
}

impl LookupClass<&StringName, Class> for Context {
    /// Look up union of GenericClass and substitute generics to yield set of classes.
    ///
    /// Substitutes all generics in the class when found.
    fn class(&self, class: &StringName, pos: Position) -> TypeResult<Class> {
        if let Some(generic_class) = self.classes.iter().find(|c| c.name.name == class.name) {
            let mut generics = HashMap::new();
            if class.name == clss::TUPLE || class.name == clss::COLLECTION {
                // Tuple and collection exception, variable generic count
                let mut generic_keys: Vec<Name> = vec![];
                for (i, gen) in enumerate(&class.generics) {
                    generic_keys.push(Name::from(format!("G{}", i).as_str()));
                    generics.insert(Name::from(format!("G{}", i).as_str()), gen.clone());
                }

                let name = StringName::new(class.name.as_str(), &generic_keys);
                let generic_class = GenericClass { name, ..generic_class.clone() };
                let class = Class::try_from((&generic_class, &generics, pos));
                return class;
            }

            let placeholders = generic_class.name.clone();

            for name in placeholders.generics.iter().zip_longest(class.generics.iter()) {
                match name {
                    EitherOrBoth::Both(placeholder, name) => {
                        generics.insert(placeholder.clone(), name.clone());
                    }
                    EitherOrBoth::Left(placeholder) => {
                        let msg = format!("No argument for generic {} in {}", placeholder, class);
                        return Err(vec![TypeErr::new(pos, &msg)]);
                    }
                    EitherOrBoth::Right(placeholder) => {
                        let msg = format!("Gave unexpected generic {} to {}", placeholder, class);
                        return Err(vec![TypeErr::new(pos, &msg)]);
                    }
                }
            }

            Class::try_from((generic_class, &generics, pos))
        } else {
            let msg = format!("Type '{}' is undefined.", class);
            Err(vec![TypeErr::new(pos, &msg)])
        }
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
    pub fn constructor(&self, without_self: bool) -> Function {
        Function {
            is_py_type: false,
            name: self.name.clone(),
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
        }
    }
}

impl GetField<Field> for Class {
    fn field(&self, name: &str, ctx: &Context, pos: Position) -> TypeResult<Field> {
        if let Some(field) = self.fields.iter().find(|f| f.name == name) {
            return Ok(field.clone());
        }

        for parent in &self.parents {
            if let Ok(ok) = ctx.class(parent, pos)?.field(name, ctx, pos) {
                return Ok(ok);
            }
        }
        Err(vec![TypeErr::new(pos, &format!("'{}' does not define '{}'", self, name))])
    }
}

impl GetFun<Function> for Class {
    /// Get function of class.
    ///
    /// If class does not implement function, traverse parents until function
    /// found.
    fn fun(&self, name: &StringName, ctx: &Context, pos: Position) -> TypeResult<Function> {
        if let Some(function) = self.functions.iter().find(|f| &f.name == name) {
            return Ok(function.clone());
        }

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
        INT => String::from(python::INT_PRIMITIVE),
        FLOAT => String::from(python::FLOAT_PRIMITIVE),
        STRING => String::from(python::STRING_PRIMITIVE),
        BOOL => String::from(python::BOOL_PRIMITIVE),
        ENUM => String::from(python::ENUM_PRIMITIVE),
        COMPLEX => String::from(python::COMPLEX_PRIMITIVE),

        COLLECTION => String::from(python::COLLECTION),
        RANGE => String::from(python::RANGE),
        SLICE => String::from(python::SLICE),
        SET => String::from(python::SET),
        LIST => String::from(python::LIST),
        TUPLE => String::from(python::TUPLE),

        CALLABLE => String::from(python::CALLABLE),
        NONE => String::from(python::NONE),
        EXCEPTION => String::from(python::EXCEPTION),
        other => String::from(other),
    }
}
