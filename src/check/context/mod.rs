use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

use crate::check::context::arg::FunctionArg;
use crate::check::context::clss::generic::GenericClass;
use crate::check::context::clss::{Class, HasParent};
use crate::check::context::field::generic::GenericField;
use crate::check::context::field::Field;
use crate::check::context::function::generic::GenericFunction;
use crate::check::context::function::Function;
use crate::check::context::generic::generics;
use crate::check::context::name::{DirectName, Name, NameUnion, NameVariant};
use crate::check::result::{TypeErr, TypeResult};
use crate::check::CheckInput;
use crate::common::delimit::comma_delm;
use crate::common::position::Position;

pub mod arg;
pub mod clss;
pub mod field;
pub mod function;
pub mod name;
mod parameter;
pub mod parent;

mod resource;
pub mod util;

mod generic;
mod python;

/// A context stores all information of all identified types of the current
/// application.
///
/// Functions and fields are also stored alongside identified classes such that
/// we can also check usage of top-level fields and functions.
#[derive(Debug)]
pub struct Context {
    classes:   HashSet<GenericClass>,
    functions: HashSet<GenericFunction>,
    fields:    HashSet<GenericField>
}

impl Context {
    pub fn class_count(&self) -> usize { self.classes.len() }

    pub fn function_count(&self) -> usize { self.functions.len() }

    pub fn field_count(&self) -> usize { self.fields.len() }
}

impl TryFrom<&[CheckInput]> for Context {
    type Error = Vec<TypeErr>;

    fn try_from(files: &[CheckInput]) -> Result<Self, Self::Error> {
        let (classes, fields, functions) = generics(files)?;
        Ok(Context { classes, functions, fields })
    }
}

pub trait LookupClass<In, Out> {
    fn class(&self, class: In, pos: &Position) -> TypeResult<Out>;
}

impl LookupClass<&DirectName, Class> for Context {
    /// Look up union of GenericClass and substitute generics to yield set of
    /// Classes.
    fn class(&self, class: &DirectName, pos: &Position) -> Result<Class, Vec<TypeErr>> {
        if let Some(generic_class) = self.classes.iter().find(|c| &c.name == class) {
            let generics = HashMap::new();
            Class::try_from((generic_class, &generics, pos))
        } else {
            let msg = format!("Class {} is undefined.", class);
            Err(vec![TypeErr::new(pos, &msg)])
        }
    }
}

impl LookupClass<&Name, ClassTuple> for Context {
    fn class(&self, class: &Name, pos: &Position) -> TypeResult<ClassTuple> {
        let variant = match &class.variant {
            NameVariant::Single(direct) => ClassVariant::Direct(self.class(direct, pos)?),
            NameVariant::Tuple(unions) => ClassVariant::Tuple(
                unions.iter().map(|union| self.class(union, pos)).collect::<Result<_, _>>()?
            ),
            NameVariant::Fun(..) => {
                let msg = format!("'{}' is not a valid class identifier", class.variant);
                return Err(vec![TypeErr::new(pos, &msg)]);
            }
        };

        Ok(ClassTuple { variant })
    }
}

impl LookupClass<&NameUnion, ClassUnion> for Context {
    /// Look up GenericClass and substitute generics to yield a Class.
    ///
    /// # Error
    ///
    /// If NameUnion is empty.
    fn class(&self, name: &NameUnion, pos: &Position) -> Result<ClassUnion, Vec<TypeErr>> {
        if name.is_empty() {
            return Err(vec![TypeErr::new(pos, &format!("UnExpected a '{}'", name))]);
        }

        let union = name.names().map(|n| self.class(&n, pos)).collect::<Result<_, _>>()?;
        Ok(ClassUnion { union })
    }
}

pub trait LookupFunction<In, Out> {
    fn function(&self, function: In, pos: &Position) -> TypeResult<Out>;
}

impl LookupFunction<&DirectName, Function> for Context {
    /// Look up a function and substitutes generics to yield a Function.
    ///
    /// If function does not exist, treat function as constructor and see if
    /// there exists a class with the same name.
    fn function(&self, function: &DirectName, pos: &Position) -> Result<Function, Vec<TypeErr>> {
        let generics = HashMap::new();

        if let Some(generic_fun) = self.functions.iter().find(|c| &c.name == function) {
            Function::try_from((generic_fun, &generics, pos))
        } else if let Some(generic_class) = self.classes.iter().find(|c| &c.name == function) {
            let class = Class::try_from((generic_class, &generics, pos))?;
            class.constructor(true)
        } else {
            let msg = format!("Function {} is undefined.", function);
            Err(vec![TypeErr::new(pos, &msg)])
        }
    }
}

pub trait LookupField<In, Out> {
    fn field(&self, field: In, pos: &Position) -> TypeResult<Out>;
}

impl LookupField<&str, Field> for Context {
    /// Look up a field and substitutes generics to yield a Field.
    fn field(&self, field: &str, pos: &Position) -> Result<Field, Vec<TypeErr>> {
        if let Some(generic_field) = self.fields.iter().find(|c| c.name == field) {
            let generics = HashMap::new();
            Field::try_from((generic_field, &generics, pos))
        } else {
            let msg = format!("Field {} is undefined.", field);
            Err(vec![TypeErr::new(pos, &msg)])
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum ClassVariant {
    Direct(Class),
    Tuple(Vec<ClassUnion>)
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ClassTuple {
    variant: ClassVariant
}

impl Display for ClassTuple {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match &self.variant {
            ClassVariant::Direct(class) => write!(f, "{}", class.name),
            ClassVariant::Tuple(classes) => {
                let names: Vec<NameUnion> = classes.iter().map(|c| c.name()).collect();
                write!(f, "({})", comma_delm(names))
            }
        }
    }
}

impl ClassTuple {
    pub fn name(&self) -> Name {
        let variant = match &self.variant {
            ClassVariant::Direct(class) => NameVariant::Single(class.name.clone()),
            ClassVariant::Tuple(classes) =>
                NameVariant::Tuple(classes.iter().map(|c| c.name()).collect()),
        };
        Name::from(&variant)
    }

    pub fn args(&self, pos: &Position) -> TypeResult<Vec<FunctionArg>> {
        match &self.variant {
            ClassVariant::Direct(class) => Ok(class.args.clone()),
            ClassVariant::Tuple(_) => {
                let msg = format!("Cannot invoke '{}' with arguments.", self);
                Err(vec![TypeErr::new(pos, &msg)])
            }
        }
    }

    pub fn fun(&self, name: &DirectName, ctx: &Context, pos: &Position) -> TypeResult<Function> {
        match &self.variant {
            ClassVariant::Direct(class) => class.fun(name, ctx, pos),
            ClassVariant::Tuple(classes) =>
                if name == &DirectName::from(function::STR) {
                    // Check that all implement __str__
                    for class in classes {
                        class.fun(name, ctx, pos)?;
                    }

                    let variant = NameVariant::Tuple(classes.iter().map(|c| c.name()).collect());
                    let self_arg = NameUnion::from(&Name::from(&variant));
                    let ret_ty = NameUnion::from(clss::STRING_PRIMITIVE);
                    Function::simple_fun(name, &self_arg, &ret_ty, pos)
                } else {
                    let msg = format!("Function '{}' undefined on '{}'", name, self);
                    Err(vec![TypeErr::new(pos, &msg)])
                },
        }
    }

    pub fn field(&self, name: &str, ctx: &Context, pos: &Position) -> TypeResult<Field> {
        match &self.variant {
            ClassVariant::Direct(class) => class.field(name, ctx, pos),
            ClassVariant::Tuple(_) => {
                let msg = format!("Attribute '{}' undefined on '{}'", name, self);
                Err(vec![TypeErr::new(pos, &msg)])
            }
        }
    }
}

#[derive(Debug, Eq)]
pub struct ClassUnion {
    union: HashSet<ClassTuple>
}

impl PartialEq for ClassUnion {
    fn eq(&self, other: &Self) -> bool {
        self.union.len() == other.union.len()
            && self.union.iter().zip(&other.union).all(|(this, that)| this == that)
    }
}

impl Hash for ClassUnion {
    fn hash<H: Hasher>(&self, state: &mut H) { self.union.iter().for_each(|c| c.hash(state)) }
}

impl HasParent<&DirectName> for ClassUnion {
    fn has_parent(
        &self,
        name: &DirectName,
        ctx: &Context,
        pos: &Position
    ) -> Result<bool, Vec<TypeErr>> {
        let res: Vec<bool> =
            self.union.iter().map(|c| c.has_parent(name, ctx, pos)).collect::<Result<_, _>>()?;
        Ok(res.iter().all(|b| *b))
    }
}

impl HasParent<&DirectName> for ClassTuple {
    fn has_parent(&self, name: &DirectName, ctx: &Context, pos: &Position) -> TypeResult<bool> {
        match &self.variant {
            ClassVariant::Direct(class) => class.has_parent(name, ctx, pos),
            ClassVariant::Tuple(_) => {
                let msg = format!("'{}' does not have parents.", self);
                Err(vec![TypeErr::new(pos, &msg)])
            }
        }
    }
}

impl HasParent<&Name> for ClassTuple {
    fn has_parent(&self, name: &Name, ctx: &Context, pos: &Position) -> TypeResult<bool> {
        match &self.variant {
            ClassVariant::Direct(class) =>
                class.has_parent(&name.as_direct("class", pos)?, ctx, pos),
            ClassVariant::Tuple(_) => {
                let msg = format!("'{}' does not have parents.", self);
                Err(vec![TypeErr::new(pos, &msg)])
            }
        }
    }
}

impl HasParent<&Name> for ClassUnion {
    fn has_parent(&self, name: &Name, ctx: &Context, pos: &Position) -> Result<bool, Vec<TypeErr>> {
        let res: Vec<bool> =
            self.union.iter().map(|c| c.has_parent(name, ctx, pos)).collect::<Result<_, _>>()?;
        Ok(res.iter().all(|b| *b))
    }
}

impl HasParent<&NameUnion> for ClassTuple {
    fn has_parent(&self, name: &NameUnion, ctx: &Context, pos: &Position) -> TypeResult<bool> {
        match &self.variant {
            ClassVariant::Direct(class) => class.has_parent(name, ctx, pos),
            ClassVariant::Tuple(_) => {
                let msg = format!("'{}' does not have parents", self);
                Err(vec![TypeErr::new(pos, &msg)])
            }
        }
    }
}

impl HasParent<&NameUnion> for ClassUnion {
    fn has_parent(
        &self,
        name: &NameUnion,
        ctx: &Context,
        pos: &Position
    ) -> Result<bool, Vec<TypeErr>> {
        let res: Vec<bool> =
            self.union.iter().map(|c| c.has_parent(name, ctx, pos)).collect::<Result<_, _>>()?;
        Ok(res.iter().all(|b| *b))
    }
}

impl ClassUnion {
    pub fn name(&self) -> NameUnion {
        let names: Vec<Name> = self.union.iter().map(|u| u.name()).collect();
        NameUnion::new(&names)
    }

    pub fn constructor(&self, pos: &Position) -> TypeResult<HashSet<Vec<FunctionArg>>> {
        // TODO check raises of constructor
        self.union.iter().map(|c| c.args(pos)).collect::<Result<_, _>>()
    }

    /// Check if ClassUnion implements a function.
    pub fn fun(&self, name: &DirectName, ctx: &Context, pos: &Position) -> TypeResult<FunUnion> {
        let union: HashSet<Function> =
            self.union.iter().map(|c| c.fun(name, ctx, pos)).collect::<Result<_, _>>()?;
        if union.is_empty() {
            let msg = format!("'{}' does not define function '{}'", self.name(), name);
            return Err(vec![TypeErr::new(pos, &msg)]);
        }

        Ok(FunUnion { union })
    }

    pub fn field(&self, name: &str, ctx: &Context, pos: &Position) -> TypeResult<FieldUnion> {
        let union: HashSet<Field> =
            self.union.iter().map(|c| c.field(name, ctx, pos)).collect::<Result<_, _>>()?;
        if union.is_empty() {
            let msg = format!("'{}' does not define attribute '{}'", self.name(), name);
            return Err(vec![TypeErr::new(pos, &msg)]);
        }

        Ok(FieldUnion { union })
    }
}

pub struct FunUnion {
    pub union: HashSet<Function>
}

pub struct FieldUnion {
    pub union: HashSet<Field>
}
