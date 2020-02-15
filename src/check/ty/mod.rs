use core::fmt;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::ops::Deref;

use crate::check::context::name::{Name, NameUnion};
use crate::check::context::{clss, Context};
use crate::check::result::{TypeErr, TypeResult};
use crate::check::ty::actual::ActualType;
use crate::check::ty::name::actual::ActualTypeName;
use crate::check::ty::name::nullable::NullableTypeName;
use crate::check::ty::nullable::NullableType;
use crate::common::delimit::comma_delimited;
use crate::common::position::Position;
use crate::parse::ast::{Node, AST};

pub mod util;

mod actual;
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
            write!(f, "{}", self.union[0])
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
            types.iter().map(NullableType::try_from).collect()?
        } else {
            HashSet::from_iter(vec![NullableType::try_from(ast)?])
        };
        Ok(Type { union })
    }
}

impl From<&str> for Type {
    fn from(name: &str) -> Type { Type::new(name, &[]) }
}

impl Type {
    pub fn new(lit: &str, generics: &[Type]) -> Type {
        let nullable_type = NullableType {
            is_nullable: lit == clss::NONE,
            actual:      ActualType::Single { name: Name::new(lit, generics) }
        };
        Type { union: HashSet::from_iter(vec![nullable]) }
    }

    pub fn union(&self, other: &Type) -> Type {
        Type { union: self.union.union(&other.union).collect() }
    }

    pub fn is_nullable(&self) -> bool { self.union.iter().all(|t| t.is_nullable) }

    pub fn is_superset(&self, other: &Type) -> bool {
        self.union.iter().all(|t| other.union.iter().all(|ot| t.is_superset(ot)))
    }

    /// Check if type implements a certain function.
    ///
    /// If type does implement function, set of all possible NameUnions returned
    /// of the function raises and return types. The first item in the tuple
    /// is what the function may raise. The second item is what the function
    /// may return.
    pub fn function(
        &self,
        args: &[Type],
        ctx: &Context,
        pos: &Position
    ) -> TypeResult<HashSet<(NameUnion, Option<NameUnion>)>> {
        unimplemented!()
    }
}
