use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;

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

impl LookupClass<&StringName, Class> for Context {
    /// Look up union of GenericClass and substitute generics to yield set of classes.
    ///
    /// Substitutes all generics in the class when found.
    fn class(&self, class: &StringName, pos: Position) -> Result<Class, Vec<TypeErr>> {
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

impl LookupClass<&TrueName, ClassVariant> for Context {
    fn class(&self, class: &TrueName, pos: Position) -> TypeResult<ClassVariant> {
        Ok(match &class.variant {
            NameVariant::Single(direct) => {
                ClassVariant::Direct(HashSet::from([self.class(direct, pos)?]))
            }
            NameVariant::Tuple(unions) => ClassVariant::Tuple(
                unions.iter().map(|union| self.class(union, pos)).collect::<Result<_, _>>()?,
            ),
            NameVariant::Fun(args, ret) => ClassVariant::Fun(
                args.iter().map(|arg| self.class(arg, pos)).collect::<Result<_, _>>()?,
                self.class(ret.deref(), pos)?,
            ),
        })
    }
}

impl LookupClass<&Name, ClassUnion> for Context {
    /// Look up GenericClass and substitute generics to field a Class.
    ///
    /// # Error
    ///
    /// If NameUnion is empty.
    fn class(&self, name: &Name, pos: Position) -> Result<ClassUnion, Vec<TypeErr>> {
        if name.is_empty() {
            return Err(vec![TypeErr::new(pos, &format!("Unexpected '{}'", name))]);
        }

        let union = name.names().map(|n| self.class(&n, pos)).collect::<Result<_, _>>()?;
        Ok(ClassUnion { union })
    }
}

pub trait LookupFunction<In, Out> {
    fn function(&self, function: In, pos: Position) -> TypeResult<Out>;
}

impl LookupFunction<&StringName, Function> for Context {
    /// Look up a function and substitutes generics to yield a Function.
    ///
    /// If function does not exist, treat function as constructor and see if
    /// there exists a class with the same truename.
    fn function(&self, function: &StringName, pos: Position) -> Result<Function, Vec<TypeErr>> {
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
    fn field(&self, field: In, pos: Position) -> TypeResult<Out>;
}

impl LookupField<&str, Field> for Context {
    /// Look up a field and substitutes generics to yield a Field.
    fn field(&self, field: &str, pos: Position) -> Result<Field, Vec<TypeErr>> {
        if let Some(generic_field) = self.fields.iter().find(|c| c.name == field) {
            let generics = HashMap::new();
            Field::try_from((generic_field, &generics, pos))
        } else {
            let msg = format!("Field {} is undefined.", field);
            Err(vec![TypeErr::new(pos, &msg)])
        }
    }
}

#[derive(Debug, Eq)]
pub enum ClassVariant {
    Direct(HashSet<Class>),
    Tuple(Vec<ClassUnion>),
    Fun(Vec<ClassUnion>, ClassUnion),
}

#[derive(Debug, Eq)]
pub struct ClassUnion {
    union: HashSet<ClassVariant>,
}

impl ClassVariant {
    pub fn field(&self, name: &str, ctx: &Context, pos: Position) -> TypeResult<FieldUnion> {
        match &self {
            ClassVariant::Direct(class_set) => {
                let fields: HashSet<Field> = class_set
                    .iter()
                    .map(|class| class.field(name, ctx, pos))
                    .collect::<Result<_, _>>()?;
                Ok(FieldUnion::from(&fields))
            }
            other => {
                let msg = format!("'{}' cannot define a field", other);
                Err(vec![TypeErr::new(pos, &msg)])
            }
        }
    }

    pub fn fun(&self, name: &StringName, ctx: &Context, pos: Position) -> TypeResult<FunUnion> {
        match &self {
            ClassVariant::Direct(class_set) => {
                let funs: HashSet<Function> = class_set
                    .iter()
                    .map(|class| class.fun(name, ctx, pos))
                    .collect::<Result<_, _>>()?;
                Ok(FunUnion::from(&funs))
            }
            other => {
                let msg = format!("'{}' cannot define a function", other);
                Err(vec![TypeErr::new(pos, &msg)])
            }
        }
    }
}

impl Display for ClassVariant {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            ClassVariant::Direct(class_set) if class_set.len() == 1 => {
                write!(f, "{}", class_set.iter().next().unwrap())
            }
            ClassVariant::Direct(class_set) => write!(f, "{{{}}}", comma_delm(class_set)),
            ClassVariant::Tuple(items) => write!(f, "({})", comma_delm(items)),
            ClassVariant::Fun(args, ret) => write!(f, "({}) -> {}", comma_delm(args), ret),
        }
    }
}

impl Display for ClassUnion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{{}}}", comma_delm(&self.union))
    }
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

impl Hash for ClassVariant {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match &self {
            ClassVariant::Direct(class_set) => {
                class_set.iter().sorted_by_key(|c| &c.name).for_each(|c| c.hash(state))
            }
            ClassVariant::Tuple(classes) => classes.hash(state),
            ClassVariant::Fun(args, ret) => {
                args.hash(state);
                ret.hash(state)
            }
        };
    }
}

impl PartialEq for ClassVariant {
    fn eq(&self, other: &Self) -> bool {
        match (&self, other) {
            (ClassVariant::Direct(l_set), ClassVariant::Direct(r_set)) => {
                let l_set: Vec<&Class> = l_set.iter().sorted_by_key(|c| &c.name).collect();
                let r_set: Vec<&Class> = r_set.iter().sorted_by_key(|c| &c.name).collect();
                l_set == r_set
            }
            (ClassVariant::Tuple(l_c), ClassVariant::Tuple(r_c)) => l_c == r_c,
            (ClassVariant::Fun(l_a, l_r), ClassVariant::Fun(r_a, r_r)) => l_a == r_a && l_r == r_r,
            _ => false,
        }
    }
}

impl HasParent<&StringName> for ClassUnion {
    fn has_parent(
        &self,
        name: &StringName,
        ctx: &Context,
        pos: Position,
    ) -> Result<bool, Vec<TypeErr>> {
        let res: Vec<bool> =
            self.union.iter().map(|c| c.has_parent(name, ctx, pos)).collect::<Result<_, _>>()?;
        Ok(res.iter().all(|b| *b))
    }
}

impl HasParent<&StringName> for ClassVariant {
    fn has_parent(&self, name: &StringName, ctx: &Context, pos: Position) -> TypeResult<bool> {
        match &self {
            ClassVariant::Direct(class_set) => {
                Ok(class_set.iter().all(|class| class.has_parent(name, ctx, pos).is_ok()))
            }
            ClassVariant::Tuple(_) | ClassVariant::Fun(..) => {
                let msg = format!("'{}' does not have parents.", self);
                Err(vec![TypeErr::new(pos, &msg)])
            }
        }
    }
}

impl HasParent<&Name> for ClassVariant {
    fn has_parent(&self, name: &Name, ctx: &Context, pos: Position) -> TypeResult<bool> {
        match &self {
            ClassVariant::Direct(union) => {
                Ok(union.iter().all(|class| class.has_parent(name, ctx, pos).is_ok()))
            }
            other => {
                let msg = format!("{} cannot have parent", other);
                Err(vec![TypeErr::new(pos, &msg)])
            }
        }
    }
}

impl HasParent<&TrueName> for ClassVariant {
    fn has_parent(&self, name: &TrueName, ctx: &Context, pos: Position) -> TypeResult<bool> {
        match &self {
            ClassVariant::Direct(union) => {
                Ok(union.iter().all(|class| class.has_parent(name, ctx, pos).is_ok()))
            }
            other => {
                let msg = format!("{} cannot have parent", other);
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
        pos: Position,
    ) -> Result<bool, Vec<TypeErr>> {
        let res: Vec<bool> =
            self.union.iter().map(|c| c.has_parent(name, ctx, pos)).collect::<Result<_, _>>()?;
        Ok(res.iter().all(|b| *b))
    }
}

impl HasParent<&Name> for ClassUnion {
    fn has_parent(&self, name: &Name, ctx: &Context, pos: Position) -> Result<bool, Vec<TypeErr>> {
        let res: Vec<bool> =
            self.union.iter().map(|c| c.has_parent(name, ctx, pos)).collect::<Result<_, _>>()?;
        Ok(res.iter().all(|b| *b))
    }
}

impl ClassUnion {
    pub fn name(&self) -> Name {
        let names: HashSet<Name> = self
            .union
            .iter()
            .map(|u| match u {
                ClassVariant::Direct(class_set) => {
                    let names: HashSet<TrueName> = class_set.iter().map(|c| c.name.clone()).collect();
                    let names: HashSet<Name> = names.iter().map(Name::from).collect();
                    Name::from(&names)
                }
                ClassVariant::Tuple(classes) => {
                    let tuple = NameVariant::Tuple(classes.iter().map(|c| c.name()).collect());
                    Name::from(&tuple)
                }
                ClassVariant::Fun(args, ret) => {
                    let args = args.iter().map(|c| c.name()).collect();
                    Name::from(&NameVariant::Fun(args, Box::from(ret.name())))
                }
            })
            .collect();

        Name::from(&names)
    }

    pub fn constructor(&self, pos: Position) -> TypeResult<HashSet<Vec<FunctionArg>>> {
        let mut fun_args: HashSet<Vec<FunctionArg>> = HashSet::new();

        let res: Vec<HashSet<Vec<FunctionArg>>> = self
            .union
            .iter()
            .map(|c| match c {
                ClassVariant::Direct(class_set) => Ok(class_set.iter().map(|c| c.args.clone()).collect()),
                other => {
                    let msg = format!("'{}' cannot have a constructor", other);
                    Err(vec![TypeErr::new(pos, &msg)])
                }
            })
            .collect::<Result<Vec<HashSet<Vec<FunctionArg>>>, _>>()?;

        res.iter().for_each(|set| {
            set.iter().for_each(|args| {
                fun_args.insert(args.clone());
            })
        });

        Ok(fun_args)
    }

    /// Check if ClassUnion implements a function.
    pub fn fun(&self, name: &StringName, ctx: &Context, pos: Position) -> TypeResult<FunUnion> {
        let union: HashSet<FunUnion> =
            self.union.iter().map(|c| c.fun(name, ctx, pos)).collect::<Result<_, _>>()?;

        if union.is_empty() {
            let msg = format!("'{}' does not define function '{}'", self.name(), name);
            return Err(vec![TypeErr::new(pos, &msg)]);
        }
        Ok(FunUnion::from(&union))
    }

    pub fn field(&self, name: &str, ctx: &Context, pos: Position) -> TypeResult<FieldUnion> {
        let union: HashSet<FieldUnion> =
            self.union.iter().map(|c| c.field(name, ctx, pos)).collect::<Result<_, _>>()?;

        if union.is_empty() {
            let msg = format!("'{}' does not define attribute '{}'", self.name(), name);
            return Err(vec![TypeErr::new(pos, &msg)]);
        }

        Ok(FieldUnion::from(&union))
    }
}

#[derive(Debug, Eq)]
pub struct FunUnion {
    pub union: HashSet<Function>,
}

impl PartialEq for FunUnion {
    fn eq(&self, other: &Self) -> bool {
        self.union.clone().iter().sorted_by_key(|f| f.name.clone()).collect::<Vec<&Function>>()
            == other.union.clone().iter().sorted_by_key(|f| f.name.clone()).collect::<Vec<&Function>>()
    }
}

impl Hash for FunUnion {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.union.iter().sorted_by_key(|f| &f.name).for_each(|f| f.hash(state))
    }
}

impl From<&HashSet<Function>> for FunUnion {
    fn from(fun_set: &HashSet<Function>) -> Self {
        FunUnion { union: fun_set.clone() }
    }
}

impl From<&HashSet<FunUnion>> for FunUnion {
    fn from(fun_set: &HashSet<FunUnion>) -> Self {
        FunUnion { union: fun_set.iter().flat_map(|f| f.union.clone()).collect() }
    }
}

impl Display for FunUnion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{{}}}", comma_delm(&self.union))
    }
}

impl FunUnion {
    fn as_direct(&self, pos: Position) -> TypeResult<Function> {
        if self.union.len() == (1_usize) {
            Ok(self.union.iter().next().unwrap().clone())
        } else {
            let msg = format!("Expected single function but was {}", &self);
            Err(vec![TypeErr::new(pos, &msg)])
        }
    }
}

#[derive(Debug, Eq)]
pub struct FieldUnion {
    pub union: HashSet<Field>,
}

impl PartialEq for FieldUnion {
    fn eq(&self, other: &Self) -> bool {
        self.union.clone().into_iter().sorted_by_key(|f| f.name.clone()).collect::<Vec<Field>>()
            == other.union.clone().into_iter().sorted_by_key(|f| f.name.clone()).collect::<Vec<Field>>()
    }
}

impl Hash for FieldUnion {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.union.iter().sorted_by_key(|f| &f.name).for_each(|f| f.hash(state))
    }
}

impl From<&HashSet<Field>> for FieldUnion {
    fn from(field_set: &HashSet<Field>) -> Self {
        FieldUnion { union: field_set.clone() }
    }
}

impl From<&HashSet<FieldUnion>> for FieldUnion {
    fn from(field_set: &HashSet<FieldUnion>) -> Self {
        FieldUnion { union: field_set.iter().flat_map(|f| f.union.clone()).collect() }
    }
}

impl Display for FieldUnion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{{}}}", comma_delm(&self.union))
    }
}

impl FieldUnion {
    fn as_direct(&self, pos: Position) -> TypeResult<Field> {
        if self.union.len() == (1_usize) {
            Ok(self.union.iter().next().unwrap().clone())
        } else {
            let msg = format!("Expected single field but was {}", &self);
            Err(vec![TypeErr::new(pos, &msg)])
        }
    }
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
        let generics = &[Name::from("Custom")];
        let list_type = StringName::new("List", generics);
        let ctx = Context::default().into_with_primitives().unwrap();

        let pos = Position::default();
        let clss = ctx.class(&list_type, pos)?;
        assert_eq!(clss.name, TrueName::from(&list_type));

        let iter_name = clss.fun(&StringName::from("__iter__"), &ctx, pos)?.ret_ty;
        for name in iter_name.as_direct("iterator", pos)? {
            let iter_class = ctx.class(&name, pos)?;
            let next_ty = iter_class.fun(&StringName::from("__next__"), &ctx, pos)?.ret_ty;
            assert_eq!(next_ty, Name::from("Custom"))
        }

        Ok(())
    }

    #[test]
    pub fn lookup_custom_set_type() -> TypeResult<()> {
        let generics = &[Name::from("Custom")];
        let list_type = StringName::new("Set", generics);
        let ctx = Context::default().into_with_primitives().unwrap();

        let pos = Position::default();
        let clss = ctx.class(&list_type, pos)?;
        assert_eq!(clss.name, TrueName::from(&list_type));

        let iter_name = clss.fun(&StringName::from("__iter__"), &ctx, pos)?.ret_ty;
        for name in iter_name.as_direct("iterator", pos)? {
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
