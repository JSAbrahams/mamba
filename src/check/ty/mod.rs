use core::fmt;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::ops::Deref;

use crate::check::context::clss::Class;
use crate::check::context::name::{Name, NameUnion};
use crate::check::context::{clss, Context};
use crate::check::result::{TypeErr, TypeResult};
use crate::check::ty::nullable::NullableType;
use crate::common::delimit::comma_delimited;
use crate::common::position::Position;
use crate::parse::ast::{Node, AST};

pub mod util;

mod nullable;

/// A Type is the actual type of an expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type {
    union: HashSet<NullableType>
}

impl Hash for Type {
    fn hash<H: Hasher>(&self, state: &mut H) { self.union.iter().for_each(|ty| ty.hash(state)) }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.union.is_empty() {
            write!(f, "()")
        } else if self.union.len() == 1 {
            write!(f, "{}", self.union.iter().next().unwrap())
        } else {
            write!(f, "{{{}}}", comma_delimited(self.union))
        }
    }
}

impl TryFrom<&Box<AST>> for Type {
    type Error = Vec<TypeErr>;

    fn try_from(value: &Box<AST>) -> Result<Self, Self::Error> { Type::try_from(value.deref()) }
}

impl TryFrom<&AST> for Type {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> TypeResult<Type> {
        let union = if let Node::TypeUnion { types } = &ast.node {
            types.iter().map(NullableType::try_from).collect::<Result<_, _>>()?
        } else {
            HashSet::from_iter(vec![NullableType::try_from(ast)?])
        };
        Ok(Type { union })
    }
}

impl From<&str> for Type {
    fn from(name: &str) -> Type { Type::new(name, &[]) }
}

impl From<&HashSet<Class>> for Type {
    fn from(classes: &HashSet<Class>) -> Self {
        let union = classes
            .iter()
            .map(|c| NullableType { is_nullable: false, name: c.name.clone() })
            .collect();
        Type { union }
    }
}

impl From<&Class> for Type {
    fn from(class: &Class) -> Self {
        let nullable = NullableType { is_nullable: false, name: class.name.clone() };
        Type { union: HashSet::from_iter(vec![nullable]) }
    }
}

impl Type {
    pub fn new(lit: &str, generics: &[Name]) -> Type {
        let is_nullable = lit == clss::NONE;
        let nullable = NullableType { is_nullable, name: Name::new(lit, generics) };
        Type { union: HashSet::from_iter(vec![nullable]) }
    }

    pub fn union(&self, other: &Type) -> Type {
        Type { union: self.union.union(&other.union).cloned().collect() }
    }

    pub fn is_nullable(&self) -> bool { self.union.iter().all(|t| t.is_nullable) }

    pub fn is_superset(&self, other: &Type) -> bool {
        self.union.iter().all(|t| other.union.iter().all(|ot| t.is_superset(ot)))
    }

    /// Check if type implements a certain function.
    ///
    /// If every type in union implements function, set of possible raises and
    /// returns type pairs given.
    pub fn function(
        &self,
        name: &Name,
        args: &[Type],
        ctx: &Context,
        pos: &Position
    ) -> TypeResult<HashSet<(Type, Option<Type>)>> {
        let res: HashSet<(NameUnion, Option<NameUnion>)> = self
            .union
            .iter()
            .map(|ty| ty.function(name, args, ctx, pos))
            .collect::<Result<_, _>>()?;

        let mut types = HashSet::new();
        for (raises, ret_ty) in res {
            let raises = Type::from(&ctx.lookup_union(&raises, pos)?);
            types.insert((
                raises,
                if let Some(ret_ty) = ret_ty {
                    Some(Type::from(&ctx.lookup_union(&ret_ty, pos)?))
                } else {
                    None
                }
            ))
        }
        Ok(types)
    }

    /// Check if Type is itself an anonymous function.
    ///
    /// If every type in union is an anonymous function with the given
    /// arguments, set of all possible return types given.
    pub fn anon_function(
        &self,
        args: &[Type],
        ctx: &Context,
        pos: &Position
    ) -> TypeResult<HashSet<Type>> {
        let res: HashSet<NameUnion> = self
            .union
            .iter()
            .map(|ty| ty.anon_function(args, ctx, pos))
            .collect::<Result<_, _>>()?;

        let mut types = HashSet::new();
        for ret_ty in res {
            types.insert(Type::from(&ctx.lookup_union(&ret_ty, pos)?))
        }
        Ok(types)
    }

    /// Check if a Type has a given field.
    ///
    /// If every type in union implements given field, set of field types given.
    pub fn field(
        &self,
        name: &Name,
        ctx: &Context,
        pos: &Position
    ) -> TypeResult<HashSet<Option<Type>>> {
        let types: HashSet<NameUnion> =
            self.union.iter().map(|ty| ty.field(name, ctx, pos)).collect::<Result<_, _>>()?;

        let mut types = HashSet::new();
        for ty in types {
            types.insert(if let Some(ty) = ty {
                Some(Type::from(&ctx.lookup_union(&ty, pos)?))
            } else {
                None
            })
        }
        Ok(types)
    }
}
