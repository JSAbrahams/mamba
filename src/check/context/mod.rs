use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

use itertools::{EitherOrBoth, Itertools};

use crate::check::context::arg::FunctionArg;
use crate::check::context::clss::{Class, HasParent};
use crate::check::context::clss::generic::GenericClass;
use crate::check::context::field::Field;
use crate::check::context::field::generic::GenericField;
use crate::check::context::function::Function;
use crate::check::context::function::generic::GenericFunction;
use crate::check::context::generic::generics;
use crate::check::name::Name;
use crate::check::name::namevariant::NameVariant;
use crate::check::name::stringname::StringName;
use crate::check::name::truename::TrueName;
use crate::check::result::{TypeErr, TypeResult};
use crate::common::delimit::comma_delm;
use crate::common::position::Position;
use crate::parse::ast::AST;

pub mod arg;
pub mod clss;
pub mod field;
pub mod function;
mod parameter;
pub mod parent;

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
    classes: HashSet<GenericClass>,
    functions: HashSet<GenericFunction>,
    fields: HashSet<GenericField>,
}

impl Context {
    pub fn class_count(&self) -> usize {
        self.classes.len()
    }

    pub fn function_count(&self) -> usize {
        self.functions.len()
    }

    pub fn field_count(&self) -> usize {
        self.fields.len()
    }
}

impl TryFrom<&[AST]> for Context {
    type Error = Vec<TypeErr>;

    fn try_from(files: &[AST]) -> Result<Self, Self::Error> {
        let (classes, fields, functions) = generics(files)?;
        Context { classes, functions, fields }.into_with_primitives()?.into_with_std_lib()
    }
}

pub trait LookupClass<In, Out> {
    fn class(&self, class: In, pos: &Position) -> TypeResult<Out>;
}

impl LookupClass<&StringName, Class> for Context {
    /// Look up union of GenericClass and substitute generics to yield set of classes.
    ///
    /// Substitutes all generics in the class when found.
    fn class(&self, class: &StringName, pos: &Position) -> Result<Class, Vec<TypeErr>> {
        if let Some(generic_class) = self.classes.iter().find(|c| {
            c.name.as_direct("Class name", pos).map(|name| name.name) == Ok(class.name.clone())
        }) {
            let mut generics = HashMap::new();
            let placeholders = generic_class.name.as_direct("Class name invalid", pos)?;

            for name in placeholders.generics.iter().zip_longest(class.generics.iter()) {
                match name {
                    EitherOrBoth::Both(placeholder, name) => {
                        generics.insert(placeholder.clone(), name.clone());
                    }
                    EitherOrBoth::Left(placeholder) => {
                        let msg = format!("No argument for generic: {}", placeholder);
                        return Err(vec![TypeErr::new(pos, &msg)]);
                    }
                    EitherOrBoth::Right(placeholder) => {
                        let msg = format!("No generic for argument: {}", placeholder);
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

impl LookupClass<&TrueName, ClassTuple> for Context {
    fn class(&self, class: &TrueName, pos: &Position) -> TypeResult<ClassTuple> {
        let variant = match &class.variant {
            NameVariant::Single(direct) => ClassVariant::Direct(self.class(direct, pos)?),
            NameVariant::Tuple(unions) => ClassVariant::Tuple(
                unions.iter().map(|union| self.class(union, pos)).collect::<Result<_, _>>()?,
            ),
            NameVariant::Fun(..) => {
                let msg = format!("'{}' is not a valid class identifier", class.variant);
                return Err(vec![TypeErr::new(pos, &msg)]);
            }
        };

        Ok(ClassTuple { variant })
    }
}

impl LookupClass<&Name, ClassUnion> for Context {
    /// Look up GenericClass and substitute generics to field a Class.
    ///
    /// # Error
    ///
    /// If NameUnion is empty.
    fn class(&self, name: &Name, pos: &Position) -> Result<ClassUnion, Vec<TypeErr>> {
        if name.is_empty() {
            return Err(vec![TypeErr::new(pos, &format!("Unexpected '{}'", name))]);
        }

        let union = name.names().map(|n| self.class(&n, pos)).collect::<Result<_, _>>()?;
        Ok(ClassUnion { union })
    }
}

pub trait LookupFunction<In, Out> {
    fn function(&self, function: In, pos: &Position) -> TypeResult<Out>;
}

impl LookupFunction<&StringName, Function> for Context {
    /// Look up a function and substitutes generics to yield a Function.
    ///
    /// If function does not exist, treat function as constructor and see if
    /// there exists a class with the same truename.
    fn function(&self, function: &StringName, pos: &Position) -> Result<Function, Vec<TypeErr>> {
        let generics = HashMap::new();

        if let Some(generic_fun) = self.functions.iter().find(|c| &c.name == function) {
            Function::try_from((generic_fun, &generics, pos))
        } else if let Some(generic_class) = self.classes.iter().find(|c| &c.name == function) {
            let class = Class::try_from((generic_class, &generics, pos))?;
            class.constructor(true, pos)
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
    Tuple(Vec<ClassUnion>),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ClassTuple {
    variant: ClassVariant,
}

impl Display for ClassTuple {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match &self.variant {
            ClassVariant::Direct(class) => write!(f, "{}", class.name),
            ClassVariant::Tuple(classes) => {
                let names: Vec<Name> = classes.iter().map(|c| c.name()).collect();
                write!(f, "({})", comma_delm(names))
            }
        }
    }
}

impl ClassTuple {
    pub fn as_direct(&self, pos: &Position) -> TypeResult<Class> {
        match &self.variant {
            ClassVariant::Direct(class) => Ok(class.clone()),
            _ => Err(vec![TypeErr::new(pos, &String::from("Expected a single class."))]),
        }
    }

    pub fn name(&self) -> TrueName {
        let variant = match &self.variant {
            ClassVariant::Direct(class) => return class.name.clone(),
            ClassVariant::Tuple(classes) => {
                NameVariant::Tuple(classes.iter().map(|c| c.name()).collect())
            }
        };
        TrueName::from(&variant)
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

    pub fn fun(&self, name: &StringName, ctx: &Context, pos: &Position) -> TypeResult<Function> {
        match &self.variant {
            ClassVariant::Direct(class) => class.fun(name, ctx, pos),
            ClassVariant::Tuple(classes) => {
                if name == &StringName::from(function::STR) {
                    // Check that all implement __str__
                    for class in classes {
                        class.fun(name, ctx, pos)?;
                    }

                    let variant = NameVariant::Tuple(classes.iter().map(|c| c.name()).collect());
                    let self_arg = Name::from(&TrueName::from(&variant));
                    let ret_ty = Name::from(clss::STRING_PRIMITIVE);
                    Function::simple_fun(name, &self_arg, &ret_ty, pos)
                } else {
                    let msg = format!("Function '{}' undefined on '{}'", name, self);
                    Err(vec![TypeErr::new(pos, &msg)])
                }
            }
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
    union: HashSet<ClassTuple>,
}

impl PartialEq for ClassUnion {
    fn eq(&self, other: &Self) -> bool {
        self.union.len() == other.union.len()
            && self.union.iter().zip(&other.union).all(|(this, that)| this == that)
    }
}

impl Hash for ClassUnion {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.union.iter().for_each(|c| c.hash(state))
    }
}

impl HasParent<&StringName> for ClassUnion {
    fn has_parent(
        &self,
        name: &StringName,
        ctx: &Context,
        pos: &Position,
    ) -> Result<bool, Vec<TypeErr>> {
        let res: Vec<bool> =
            self.union.iter().map(|c| c.has_parent(name, ctx, pos)).collect::<Result<_, _>>()?;
        Ok(res.iter().all(|b| *b))
    }
}

impl HasParent<&StringName> for ClassTuple {
    fn has_parent(&self, name: &StringName, ctx: &Context, pos: &Position) -> TypeResult<bool> {
        match &self.variant {
            ClassVariant::Direct(class) => class.has_parent(name, ctx, pos),
            ClassVariant::Tuple(_) => {
                let msg = format!("'{}' does not have parents.", self);
                Err(vec![TypeErr::new(pos, &msg)])
            }
        }
    }
}

impl HasParent<&TrueName> for ClassTuple {
    fn has_parent(&self, name: &TrueName, ctx: &Context, pos: &Position) -> TypeResult<bool> {
        match &self.variant {
            ClassVariant::Direct(class) => {
                class.has_parent(&name.as_direct("class", pos)?, ctx, pos)
            }
            ClassVariant::Tuple(_) => {
                let msg = format!("'{}' does not have parents.", self);
                Err(vec![TypeErr::new(pos, &msg)])
            }
        }
    }
}

impl HasParent<&TrueName> for ClassUnion {
    fn has_parent(
        &self,
        name: &TrueName,
        ctx: &Context,
        pos: &Position,
    ) -> Result<bool, Vec<TypeErr>> {
        let res: Vec<bool> =
            self.union.iter().map(|c| c.has_parent(name, ctx, pos)).collect::<Result<_, _>>()?;
        Ok(res.iter().all(|b| *b))
    }
}

impl HasParent<&Name> for ClassTuple {
    fn has_parent(&self, name: &Name, ctx: &Context, pos: &Position) -> TypeResult<bool> {
        match &self.variant {
            ClassVariant::Direct(class) => class.has_parent(name, ctx, pos),
            ClassVariant::Tuple(_) => {
                let msg = format!("'{}' does not have parents", self);
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

impl ClassUnion {
    pub fn name(&self) -> Name {
        let names: Vec<TrueName> = self.union.iter().map(|u| u.name()).collect();
        Name::new(&names)
    }

    pub fn constructor(&self, pos: &Position) -> TypeResult<HashSet<Vec<FunctionArg>>> {
        // TODO check raises of constructor
        self.union.iter().map(|c| c.args(pos)).collect::<Result<_, _>>()
    }

    /// Check if ClassUnion implements a function.
    pub fn fun(&self, name: &StringName, ctx: &Context, pos: &Position) -> TypeResult<FunUnion> {
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
    pub union: HashSet<Function>,
}

#[derive(Debug)]
pub struct FieldUnion {
    pub union: HashSet<Field>,
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use crate::check::context::{Context, LookupClass};
    use crate::check::name::Name;
    use crate::check::name::stringname::StringName;
    use crate::check::name::truename::TrueName;
    use crate::check::result::TypeResult;
    use crate::common::position::Position;

    #[test]
    pub fn lookup_custom_list_type() -> TypeResult<()> {
        let generics = &[Name::from("Int")];
        let list_type = StringName::new("List", generics);
        let ctx = Context::default().into_with_primitives().unwrap();

        let clss = ctx.class(&list_type, &Position::default())?;
        assert_eq!(clss.name, TrueName::from(&list_type));
        Ok(())
    }

    #[test]
    pub fn primitives_present() {
        let files = vec![];
        let context = Context::try_from(files.as_slice()).unwrap();
        let context = context.into_with_primitives().unwrap();

        context.class(&StringName::from("String"), &Position::default()).unwrap();
        context.class(&StringName::from("Bool"), &Position::default()).unwrap();
        context.class(&StringName::from("Float"), &Position::default()).unwrap();
        context.class(&StringName::from("Int"), &Position::default()).unwrap();
        context.class(&StringName::from("Complex"), &Position::default()).unwrap();
    }

    #[test]
    pub fn std_lib_present() {
        let files = vec![];
        let context = Context::try_from(files.as_slice()).unwrap();
        let context = context.into_with_std_lib().unwrap();

        context.class(&StringName::from("Range"), &Position::default()).unwrap();
        context.class(&StringName::from("None"), &Position::default()).unwrap();
        context.class(&StringName::from("Exception"), &Position::default()).unwrap();
    }
}
