use std::collections::HashSet;
use std::convert::TryFrom;

use crate::check::context::clss::generic::GenericClass;
use crate::check::context::field::generic::GenericField;
use crate::check::context::function::generic::GenericFunction;
use crate::check::context::generic::generics;
use crate::check::name::Any;
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
#[derive(Debug)]
pub struct Context {
    pub classes: HashSet<GenericClass>,
    pub functions: HashSet<GenericFunction>,
    pub fields: HashSet<GenericField>,
}

impl Default for Context {
    /// Create default Context with only `Any` type present.
    fn default() -> Self {
        let mut classes = HashSet::new();
        classes.insert(GenericClass::any());

        Context { classes, functions: Default::default(), fields: Default::default() }
    }
}

impl TryFrom<&[AST]> for Context {
    type Error = Vec<TypeErr>;

    fn try_from(files: &[AST]) -> Result<Self, Self::Error> {
        let (classes, fields, functions) = generics(files)?;
        let mut context = Context::default();
        classes.iter().for_each(|clss| {
            context.classes.insert(clss.clone());
        });
        fields.iter().for_each(|fld| {
            context.fields.insert(fld.clone());
        });
        functions.iter().for_each(|func| {
            context.functions.insert(func.clone());
        });

        context.into_with_primitives()?.into_with_std_lib()
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

    use crate::check::context::clss::GetFun;
    use crate::check::context::{Context, LookupClass, LookupFunction};
    use crate::check::name::string_name::StringName;
    use crate::check::name::{Name, Union};
    use crate::check::result::TypeResult;
    use crate::common::position::Position;
    use crate::parse::parse;

    #[test]
    pub fn lookup_custom_list_type() -> TypeResult<()> {
        let generics = &[Name::from("Custom")];
        let list_type = StringName::new("List", generics);
        let ctx = Context::default().into_with_primitives().unwrap();

        let pos = Position::invisible();
        let clss = ctx.class(&list_type, pos)?;
        assert_eq!(clss.name, list_type);

        let iter_name = clss.fun(&StringName::from("__iter__"), pos)?.ret_ty;
        for name in iter_name.as_direct() {
            let iter_class = ctx.class(&name, pos)?;
            let next_ty = iter_class.fun(&StringName::from("__next__"), pos)?.ret_ty;
            assert_eq!(next_ty, Name::from("Custom"))
        }

        Ok(())
    }

    #[test]
    pub fn lookup_custom_set_type() -> TypeResult<()> {
        let generics = &[Name::from("Custom")];
        let set_type = StringName::new("Set", generics);
        let ctx = Context::default().into_with_primitives().unwrap();

        let pos = Position::invisible();
        let clss = ctx.class(&set_type, pos)?;
        assert_eq!(clss.name, set_type);

        let iter_name = clss.fun(&StringName::from("__iter__"), pos)?.ret_ty;
        for name in iter_name.as_direct() {
            let iter_class = ctx.class(&name, pos)?;
            let next_ty = iter_class.fun(&StringName::from("__next__"), pos)?.ret_ty;
            assert_eq!(next_ty, Name::from("Custom"))
        }

        Ok(())
    }

    #[test]
    pub fn default_any_present() {
        let files = vec![];
        let context = Context::try_from(files.as_slice()).unwrap();

        context.class(&StringName::from("Any"), Position::invisible()).unwrap();
    }

    #[test]
    pub fn primitives_present() {
        let files = vec![];
        let context = Context::try_from(files.as_slice()).unwrap();
        let context = context.into_with_primitives().unwrap();

        context.class(&StringName::from("Str"), Position::invisible()).unwrap();
        context.class(&StringName::from("Bool"), Position::invisible()).unwrap();
        context.class(&StringName::from("Float"), Position::invisible()).unwrap();
        context.class(&StringName::from("Int"), Position::invisible()).unwrap();
        context.class(&StringName::from("Complex"), Position::invisible()).unwrap();
    }

    #[test]
    pub fn int_constructor() {
        let files = vec![];
        let context = Context::try_from(files.as_slice()).unwrap();
        let context = context.into_with_primitives().unwrap();

        let int_class = context.class(&StringName::from("Int"), Position::invisible()).unwrap();
        let args = int_class.args;

        assert_eq!(args.len(), 2);
        assert_eq!(args[0].ty.clone().unwrap(), Name::from("Int"));
        assert_eq!(
            args[1].ty.clone().unwrap(),
            Name::from("Int").union(&Name::from("Str")).union(&Name::from("Float"))
        );
    }

    #[test]
    pub fn int_function_defined() {
        let files = vec![];
        let context = Context::try_from(files.as_slice()).unwrap();
        let context = context.into_with_primitives().unwrap();

        let int_fun = context.function(&StringName::from("Int"), Position::invisible()).unwrap();

        assert_eq!(int_fun.arguments.len(), 1);
        assert_eq!(
            int_fun.arguments[0].ty.clone().unwrap(),
            Name::from("Int").union(&Name::from("Str")).union(&Name::from("Float"))
        );

        assert_eq!(int_fun.ret_ty, Name::from("Int"))
    }

    #[test]
    pub fn std_lib_present() {
        let files = vec![];
        let context = Context::try_from(files.as_slice()).unwrap();
        let context = context.into_with_std_lib().unwrap();

        context.class(&StringName::from("Range"), Position::invisible()).unwrap();
        context.class(&StringName::from("None"), Position::invisible()).unwrap();
        context.class(&StringName::from("Exception"), Position::invisible()).unwrap();
    }

    #[test]
    pub fn test_import_parse() {
        let file = parse("import IPv4Address").unwrap();
        let context = Context::try_from(vec![*file.clone()].as_slice()).unwrap();

        context.class(&StringName::from("IPv4Address"), Position::invisible()).unwrap();
    }

    #[test]
    pub fn test_import_as_parse() {
        let file = parse("import IPv4Address as Other").unwrap();
        let context = Context::try_from(vec![*file.clone()].as_slice()).unwrap();

        context.class(&StringName::from("Other"), Position::invisible()).unwrap();
    }

    #[test]
    pub fn test_import_as_too_many_parse() {
        let file = parse("import IPv4Address as Other, Other2").unwrap();
        Context::try_from(vec![*file.clone()].as_slice()).unwrap_err();
    }

    #[test]
    pub fn test_from_import_parse() {
        let file = parse("from ipaddress import IPv4Address").unwrap();
        let context = Context::try_from(vec![*file.clone()].as_slice()).unwrap();

        context.class(&StringName::from("IPv4Address"), Position::invisible()).unwrap();
    }

    #[test]
    pub fn test_from_import_as_parse() {
        let file = parse("from ipaddress import IPv4Address as Other").unwrap();
        let context = Context::try_from(vec![*file.clone()].as_slice()).unwrap();

        context.class(&StringName::from("Other"), Position::invisible()).unwrap();
    }

    #[test]
    pub fn tuple_argument() {
        let file = parse("def f(b: (Int, Int))").unwrap();
        let context = Context::try_from(vec![*file.clone()].as_slice()).unwrap();

        let f = context
            .function(&StringName::from("f"), Position::invisible())
            .expect("function exists");
        let arg = f.arguments.first().expect("first argument").clone();
        let arg_ty = arg.ty.expect("argument has type").clone();

        let tuple = StringName::new("Tuple", &[Name::from("Int"), Name::from("Int")]);
        assert_eq!(arg_ty, Name::from(&tuple));
    }
}
