use core::fmt;
use std::borrow::BorrowMut;
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;

use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::context::type_name::actual::ActualTypeName;
use crate::type_checker::context::type_name::nullable::NullableTypeName;
use crate::type_checker::environment::expression_type::ExpressionType;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use crate::type_checker::util::comma_delimited;

pub mod actual;
pub mod nullable;
pub mod python;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TypeName {
    Single { ty: NullableTypeName },
    Union { union: HashSet<NullableTypeName> }
}

impl Hash for TypeName {
    /// Hash TypeName
    ///
    /// As a TypeName may be a union, the runtime is O(n) instead of O(1)
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            TypeName::Single { ty } => ty.hash(state),
            TypeName::Union { union } => union.iter().for_each(|ty| ty.hash(state))
        }
    }
}

impl Display for TypeName {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            TypeName::Single { ty } => write!(f, "{}", ty),
            TypeName::Union { union } => write!(f, "{{{}}}", comma_delimited(union))
        }
    }
}

impl TryFrom<&AST> for TypeName {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> TypeResult<TypeName> {
        if let Node::TypeUnion { types } = &ast.node {
            let (types, errs): (Vec<_>, Vec<_>) =
                types.iter().map(NullableTypeName::try_from).partition(Result::is_ok);
            if errs.is_empty() {
                Ok(TypeName::Union { union: types.into_iter().map(Result::unwrap).collect() })
            } else {
                Err(errs.into_iter().map(Result::unwrap_err).flatten().collect())
            }
        } else {
            Ok(TypeName::Single { ty: NullableTypeName::try_from(ast)? })
        }
    }
}

impl From<&str> for TypeName {
    fn from(name: &str) -> TypeName { TypeName::new(name, &vec![]) }
}

impl From<&ActualTypeName> for TypeName {
    fn from(actual_type_name: &ActualTypeName) -> TypeName {
        TypeName::Single {
            ty: NullableTypeName { is_nullable: false, actual: actual_type_name.clone() }
        }
    }
}

impl From<&ExpressionType> for TypeName {
    fn from(expression_type: &ExpressionType) -> TypeName {
        match &expression_type {
            ExpressionType::Single { ty } => TypeName::Single {
                ty: NullableTypeName {
                    is_nullable: false,
                    actual:      ActualTypeName::from(&ty.actual_ty())
                }
            },
            ExpressionType::Union { union } => TypeName::Union {
                union: union.iter().map(|ty| NullableTypeName::from(ty)).collect()
            }
        }
    }
}

impl TypeName {
    pub fn new(lit: &str, generics: &[TypeName]) -> TypeName {
        TypeName::Single {
            ty: NullableTypeName {
                is_nullable: false,
                actual:      ActualTypeName::Single {
                    lit:      String::from(lit),
                    generics: Vec::from(generics)
                }
            }
        }
    }

    pub fn names(&self) -> HashSet<ActualTypeName> {
        match &self {
            TypeName::Single { ty } => HashSet::from_iter(vec![ty.actual.clone()].into_iter()),
            TypeName::Union { union } => union.into_iter().map(|ty| ty.actual.clone()).collect()
        }
    }

    pub fn union(self, other: &TypeName) -> TypeName {
        match (self.clone().borrow_mut(), other.clone().borrow_mut()) {
            (TypeName::Single { ty }, TypeName::Single { ty: o_ty }) => TypeName::Union {
                union: HashSet::from_iter(vec![ty.clone(), o_ty.clone()].into_iter())
            },
            (TypeName::Union { union }, TypeName::Union { union: o_union }) =>
                TypeName::Union { union: union.union(o_union).cloned().collect() },
            (TypeName::Single { ty }, TypeName::Union { union })
            | (TypeName::Union { union }, TypeName::Single { ty }) => {
                union.insert(ty.clone());
                TypeName::Union { union: union.clone() }
            }
        }
    }

    pub fn single(self, pos: &Position) -> TypeResult<ActualTypeName> {
        match self {
            TypeName::Single { ty } => Ok(ty.actual),
            _ => Err(vec![TypeErr::new(pos, "Unions not supported here")])
        }
    }

    pub fn substitute(
        &self,
        generics: &HashMap<String, TypeName>,
        pos: &Position
    ) -> TypeResult<TypeName> {
        Ok(match self {
            TypeName::Single { ty } => TypeName::Single { ty: ty.substitute(generics, pos)? },
            TypeName::Union { union } => TypeName::Union {
                union: union
                    .into_iter()
                    .map(|ty| ty.substitute(generics, pos))
                    .collect::<Result<_, _>>()?
            }
        })
    }
}
