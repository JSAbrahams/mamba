use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;

use crate::check::context::clss::generic::GenericClass;
use crate::check::context::clss::Class;
use crate::check::context::field::generic::GenericField;
use crate::check::context::field::Field;
use crate::check::context::function::generic::GenericFunction;
use crate::check::context::function::Function;
use crate::check::context::generic::generics;
use crate::check::context::name::Name;
use crate::check::result::{TypeErr, TypeResult};
use crate::check::ty::name::TypeName;
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

impl Context {
    /// Look up GenericClass and substitute generics to yield a Class
    pub fn lookup_class(&self, name: &Name, pos: &Position) -> TypeResult<Class> {
        if let Some(generic_class) = self.classes.iter().find(|c| c.name == name) {
            let generics = HashMap::new();
            Class::try_from((generic_class, &generics, &self.classes, pos))
        } else {
            let msg = format!("Class {} is undefined.", name);
            Err(vec![TypeErr::new(pos, &msg)])
        }
    }

    /// Look up GenericFunction and substitute generics to yield a Function.
    pub fn lookup_fun(&self, name: &Name, pos: &Position) -> TypeResult<Function> {
        if let Some(generic_fun) = self.functions.iter().find(|c| c.name == name) {
            let generics = HashMap::new();
            Function::try_from((generic_fun, &generics, pos))
        } else {
            let msg = format!("Function {} is undefined.", name);
            Err(vec![TypeErr::new(pos, &msg)])
        }
    }

    /// Look up a field and substitutes generics to yield a Field.
    pub fn lookup_field(&self, name: &Name, pos: &Position) -> TypeResult<Field> {
        if let Some(generic_field) = self.fields.iter().find(|c| c.name == name) {
            let generics = HashMap::new();
            Field::try_from((generic_field, &generics, pos))
        } else {
            let msg = format!("Field {} is undefined.", name);
            Err(vec![TypeErr::new(pos, &msg)])
        }
    }
}
