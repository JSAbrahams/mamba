use std::collections::HashSet;
use std::convert::TryFrom;

use crate::check::context::clss::generic::GenericClass;
use crate::check::context::field::generic::GenericField;
use crate::check::context::function::generic::GenericFunction;
use crate::check::context::generic::generics;
use crate::check::result::{TypeErr, TypeResult};
use crate::common::position::Position;
use crate::parse::ast::AST;

pub mod arg;
pub mod clss;
pub mod field;
pub mod function;
pub mod parent;

mod parameter;
mod resource;

mod generic;
mod python;

/// A context stores all information of all identified types of the current
/// application.
///
/// Functions and fields are also stored alongside identified classes such that
/// we can also check usage of top-level fields and functions.
#[derive(Debug, Default)]
pub struct Context {
    pub classes: HashSet<GenericClass>,
    pub functions: HashSet<GenericFunction>,
    pub fields: HashSet<GenericField>,
}

impl TryFrom<&[AST]> for Context {
    type Error = Vec<TypeErr>;

    fn try_from(files: &[AST]) -> Result<Self, Self::Error> {
        let (classes, fields, functions) = generics(files)?;
        Context { classes, functions, fields }.into_with_primitives()?.into_with_std_lib()
    }
}

pub trait LookupClass<In, Out> {
    fn class(&self, class: In, pos: Position) -> TypeResult<Out>;
}

pub trait LookupFunction<In, Out> {
    fn function(&self, function: In, pos: Position) -> TypeResult<Out>;
}

pub trait LookupField<In, Out> {
    fn field(&self, field: In, pos: Position) -> TypeResult<Out>;
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use crate::check::context::{Context, LookupClass};
    use crate::check::context::clss::GetFun;
    use crate::check::name::Name;
    use crate::check::name::string_name::StringName;
    use crate::check::result::TypeResult;
    use crate::common::position::Position;

    #[test]
    pub fn lookup_custom_list_type() -> TypeResult<()> {
        let generics = &[Name::from("Custom")];
        let list_type = StringName::new("List", generics);
        let ctx = Context::default().into_with_primitives().unwrap();

        let pos = Position::default();
        let clss = ctx.class(&list_type, pos)?;
        assert_eq!(clss.name, list_type);

        let iter_name = clss.fun(&StringName::from("__iter__"), &ctx, pos)?.ret_ty;
        for name in iter_name.as_direct() {
            let iter_class = ctx.class(&name, pos)?;
            let next_ty = iter_class.fun(&StringName::from("__next__"), &ctx, pos)?.ret_ty;
            assert_eq!(next_ty, Name::from("Custom"))
        }

        Ok(())
    }

    #[test]
    pub fn lookup_custom_set_type() -> TypeResult<()> {
        let generics = &[Name::from("Custom")];
        let set_type = StringName::new("Set", generics);
        let ctx = Context::default().into_with_primitives().unwrap();

        let pos = Position::default();
        let clss = ctx.class(&set_type, pos)?;
        assert_eq!(clss.name, set_type);

        let iter_name = clss.fun(&StringName::from("__iter__"), &ctx, pos)?.ret_ty;
        for name in iter_name.as_direct() {
            let iter_class = ctx.class(&name, pos)?;
            let next_ty = iter_class.fun(&StringName::from("__next__"), &ctx, pos)?.ret_ty;
            assert_eq!(next_ty, Name::from("Custom"))
        }

        Ok(())
    }

    #[test]
    pub fn primitives_present() {
        let files = vec![];
        let context = Context::try_from(files.as_slice()).unwrap();
        let context = context.into_with_primitives().unwrap();

        context.class(&StringName::from("String"), Position::default()).unwrap();
        context.class(&StringName::from("Bool"), Position::default()).unwrap();
        context.class(&StringName::from("Float"), Position::default()).unwrap();
        context.class(&StringName::from("Int"), Position::default()).unwrap();
        context.class(&StringName::from("Complex"), Position::default()).unwrap();
    }

    #[test]
    pub fn std_lib_present() {
        let files = vec![];
        let context = Context::try_from(files.as_slice()).unwrap();
        let context = context.into_with_std_lib().unwrap();

        context.class(&StringName::from("Range"), Position::default()).unwrap();
        context.class(&StringName::from("None"), Position::default()).unwrap();
        context.class(&StringName::from("Exception"), Position::default()).unwrap();
    }
}
