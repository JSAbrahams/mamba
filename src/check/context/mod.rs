use std::collections::hash_set::IntoIter;
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;

use crate::check::context::arg::FunctionArg;
use crate::check::context::clss::generic::GenericClass;
use crate::check::context::clss::{Class, HasParent};
use crate::check::context::field::generic::GenericField;
use crate::check::context::field::Field;
use crate::check::context::function::generic::GenericFunction;
use crate::check::context::function::Function;
use crate::check::context::generic::generics;
use crate::check::context::name::{DirectName, Name, NameUnion};
use crate::check::result::{TypeErr, TypeResult};
use crate::check::CheckInput;
use crate::common::position::Position;

pub mod arg;
pub mod clss;
pub mod field;
pub mod function;
pub mod name;
pub mod parameter;
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
            Class::try_from((generic_class, &generics, &self.classes, pos))
        } else {
            let msg = format!("Class {} is undefined.", class);
            Err(vec![TypeErr::new(pos, &msg)])
        }
    }
}

impl LookupClass<&NameUnion, ClassUnion> for Context {
    /// Look up GenericClass and substitute generics to yield a Class.
    fn class(&self, name: &NameUnion, pos: &Position) -> Result<ClassUnion, Vec<TypeErr>> {
        let union = name
            .as_direct("class", pos)?
            .iter()
            .map(|n| self.class(n, pos))
            .collect::<Result<_, _>>()?;
        Ok(ClassUnion { union })
    }
}

pub trait LookupFunction<In, Out> {
    fn function(&self, function: In, pos: &Position) -> TypeResult<Out>;
}

impl LookupFunction<&DirectName, Function> for Context {
    /// Look up a function and substitutes generics to yield a Function.
    fn function(&self, function: &DirectName, pos: &Position) -> Result<Function, Vec<TypeErr>> {
        if let Some(generic_fun) = self.functions.iter().find(|c| &c.name == function) {
            let generics = HashMap::new();
            Function::try_from((generic_fun, &generics, pos))
        } else {
            let msg = format!("Function {} is undefined.", function);
            Err(vec![TypeErr::new(pos, &msg)])
        }
    }
}

impl LookupFunction<&NameUnion, FunctionUnion> for Context {
    /// Look up a function and substitutes generics to yield a Function.
    fn function(&self, name: &NameUnion, pos: &Position) -> Result<FunctionUnion, Vec<TypeErr>> {
        let union = name
            .as_direct("function", pos)?
            .iter()
            .map(|f| self.function(f, pos))
            .collect::<Result<_, _>>()?;
        Ok(FunctionUnion { union })
    }
}

pub trait LookupField<In, Out> {
    fn field(&self, field: In, pos: &Position) -> TypeResult<Out>;
}

impl LookupField<&str, Field> for Context {
    /// Look up a field and substitutes generics to yield a Field.
    fn field(&self, field: &str, pos: &Position) -> Result<Field, Vec<TypeErr>> {
        if let Some(generic_field) = self.fields.iter().find(|c| &c.name == field) {
            let generics = HashMap::new();
            Field::try_from((generic_field, &generics, pos))
        } else {
            let msg = format!("Field {} is undefined.", field);
            Err(vec![TypeErr::new(pos, &msg)])
        }
    }
}

pub struct ClassUnion {
    union: HashSet<Class>
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
        let names: Vec<Name> = self.union.iter().map(|u| Name::from(&u.name)).collect();
        NameUnion::new(&names)
    }

    pub fn constructor(&self) -> HashSet<Vec<FunctionArg>> {
        // TODO check raises of constructor
        self.union.iter().map(|c| c.args.clone()).collect()
    }

    pub fn classes(&self) -> IntoIter<Class> { self.union.clone().into_iter() }

    pub fn function(&self, fun_name: &DirectName, pos: &Position) -> TypeResult<FunctionUnion> {
        let union =
            self.union.iter().map(|c| c.function(fun_name, pos)).collect::<Result<_, _>>()?;
        Ok(FunctionUnion { union })
    }

    pub fn field(&self, name: &str, pos: &Position) -> TypeResult<FieldUnion> {
        let union = self.union.iter().map(|c| c.field(name, pos)).collect::<Result<_, _>>()?;
        Ok(FieldUnion { union })
    }
}

pub struct FunctionUnion {
    pub union: HashSet<Function>
}

pub struct FieldUnion {
    pub union: HashSet<Field>
}
