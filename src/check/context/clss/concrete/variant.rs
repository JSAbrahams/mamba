use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::ops::Deref;

use itertools::Itertools;

use crate::{Context, TypeErr};
use crate::check::context::{clss, LookupClass};
use crate::check::context::clss::{CALLABLE, Class, GetField, GetFun, HasParent, TUPLE};
use crate::check::context::clss::concrete::union::ClassUnion;
use crate::check::context::field::concrete::union::FieldUnion;
use crate::check::context::field::Field;
use crate::check::context::function::concrete::union::FunUnion;
use crate::check::context::function::Function;
use crate::check::name::Name;
use crate::check::name::namevariant::NameVariant;
use crate::check::name::stringname::StringName;
use crate::check::name::truename::TrueName;
use crate::check::result::TypeResult;
use crate::common::position::Position;

#[derive(Debug, Eq)]
pub enum ClassVariant {
    Direct(HashSet<Class>),
    Tuple(Vec<ClassUnion>),
    Fun(Vec<ClassUnion>, ClassUnion),
}

impl From<&ClassVariant> for Name {
    fn from(class_variant: &ClassVariant) -> Self {
        let name_variant = match &class_variant {
            ClassVariant::Direct(class) if class.len() == 1 => {
                class.iter().next().unwrap().name.variant.clone()
            }
            ClassVariant::Direct(classes) => {
                let names = classes.iter().map(|class| &class.name.variant).map(Name::from);
                return Name::from(&HashSet::from_iter(names));
            }
            ClassVariant::Tuple(class_unions) => {
                NameVariant::Tuple(class_unions.iter().map(Name::from).collect())
            }
            ClassVariant::Fun(args, ret) => {
                NameVariant::Fun(args.iter().map(Name::from).collect(), Box::from(Name::from(ret)))
            }
        };
        Name::from(&name_variant)
    }
}

impl HasParent<&StringName> for ClassVariant {
    fn has_parent(&self, name: &StringName, ctx: &Context, pos: Position) -> TypeResult<bool> {
        match &self {
            ClassVariant::Direct(class_set) => Ok(class_set
                .iter()
                .map(|class| class.has_parent(name, ctx, pos))
                .collect::<Result<Vec<bool>, _>>()?
                .iter()
                .any(|b| *b)),
            ClassVariant::Tuple(_) | ClassVariant::Fun(..) => {
                let msg = format!("'{}' does not have parents.", self);
                Err(vec![TypeErr::new(pos, &msg)])
            }
        }
    }
}

impl HasParent<&Name> for ClassVariant {
    fn has_parent(&self, name: &Name, ctx: &Context, pos: Position) -> TypeResult<bool> {
        let res = name.names.iter().map(|t_name| self.has_parent(t_name, ctx, pos));
        Ok(res.collect::<Result<Vec<bool>, _>>()?.iter().any(|b| *b))
    }
}

impl HasParent<&NameVariant> for ClassVariant {
    fn has_parent(&self, name: &NameVariant, ctx: &Context, pos: Position) -> TypeResult<bool> {
        let name = match &name {
            NameVariant::Single(string_name) => string_name.clone(),
            NameVariant::Tuple(_) => StringName::from(TUPLE),
            NameVariant::Fun(..) => StringName::from(CALLABLE),
        };
        self.has_parent(&name, ctx, pos)
    }
}

impl HasParent<&TrueName> for ClassVariant {
    fn has_parent(&self, name: &TrueName, ctx: &Context, pos: Position) -> TypeResult<bool> {
        self.has_parent(&name.variant, ctx, pos)
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

impl GetField<FieldUnion> for ClassVariant {
    fn field(&self, name: &str, ctx: &Context, pos: Position) -> TypeResult<FieldUnion> {
        match &self {
            ClassVariant::Direct(class_set) => {
                let fields: HashSet<Field> = class_set
                    .iter()
                    .map(|class| class.field(name, ctx, pos))
                    .collect::<Result<_, _>>()?;
                Ok(FieldUnion::from(&fields))
            }
            ClassVariant::Tuple(_) => {
                let clss_name = StringName::from(clss::TUPLE);
                let class = HashSet::from_iter([ctx.class(&clss_name, pos)?].iter().cloned());
                ClassVariant::Direct(class).field(name, ctx, pos)
            }
            ClassVariant::Fun(..) => {
                let clss_name = StringName::from(clss::CALLABLE);
                let class = HashSet::from_iter([ctx.class(&clss_name, pos)?].iter().cloned());
                ClassVariant::Direct(class).field(name, ctx, pos)
            }
        }
    }
}

impl GetFun<FunUnion> for ClassVariant {
    fn fun(&self, name: &StringName, ctx: &Context, pos: Position) -> TypeResult<FunUnion> {
        match &self {
            ClassVariant::Direct(class_set) => {
                let funs: HashSet<Function> = class_set
                    .iter()
                    .map(|class| class.fun(name, ctx, pos))
                    .collect::<Result<_, _>>()?;
                Ok(FunUnion::from(&funs))
            }
            ClassVariant::Tuple(_) => {
                let clss_name = StringName::from(clss::TUPLE);
                let class = HashSet::from_iter([ctx.class(&clss_name, pos)?].iter().cloned());
                ClassVariant::Direct(class).fun(name, ctx, pos)
            }
            ClassVariant::Fun(..) => {
                let clss_name = StringName::from(clss::CALLABLE);
                let class = HashSet::from_iter([ctx.class(&clss_name, pos)?].iter().cloned());
                ClassVariant::Direct(class).fun(name, ctx, pos)
            }
        }
    }
}

impl Display for ClassVariant {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Name::from(self))
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
