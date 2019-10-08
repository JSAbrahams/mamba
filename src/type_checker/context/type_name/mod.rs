use core::fmt;
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};

use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::context::type_name::actual::ActualTypeName;
use crate::type_checker::environment::expression_type::ExpressionType;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use std::hash::{Hash, Hasher};

pub mod actual;
pub mod python;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TypeName {
    Single { ty: ActualTypeName },
    Union { union: HashSet<ActualTypeName> }
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
            TypeName::Union { union } => write!(f, "{{{:#?}}}", union)
        }
    }
}

impl From<&ExpressionType> for TypeName {
    fn from(expr_ty: &ExpressionType) -> Self {
        match expr_ty {
            ExpressionType::Single { mut_ty } =>
                TypeName::from(&ActualTypeName::from(&mut_ty.actual_ty)),
            ExpressionType::Union { union } => {
                let union =
                    union.iter().map(|expr_ty| ActualTypeName::from(&expr_ty.actual_ty)).collect();
                TypeName::Union { union }
            }
        }
    }
}

impl From<&ActualTypeName> for TypeName {
    fn from(actual: &ActualTypeName) -> Self { TypeName::Single { ty: actual.clone() } }
}

impl From<&str> for TypeName {
    fn from(name: &str) -> Self { TypeName::Single { ty: ActualTypeName::from(name) } }
}

impl TryFrom<&AST> for TypeName {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> TypeResult<TypeName> {
        if let Node::TypeUnion { types } = &ast.node {
            let (types, errs): (Vec<_>, Vec<_>) =
                types.iter().map(ActualTypeName::try_from).partition(Result::is_ok);
            if errs.is_empty() {
                Ok(TypeName::Union { union: types.into_iter().map(Result::unwrap).collect() })
            } else {
                Err(errs.into_iter().map(Result::unwrap_err).flatten().collect())
            }
        } else {
            Ok(TypeName::Single { ty: ActualTypeName::try_from(ast)? })
        }
    }
}

impl TypeName {
    pub fn new(lit: &str, generics: &[ActualTypeName]) -> TypeName {
        TypeName::Single {
            ty: ActualTypeName::Single {
                lit:      String::from(lit),
                generics: Vec::from(generics)
            }
        }
    }

    pub fn single(&self, pos: &Position) -> TypeResult<ActualTypeName> {
        match self {
            TypeName::Single { ty } => Ok(ty.clone()),
            TypeName::Union { .. } => Err(vec![TypeErr::new(pos, "Unions not supported here")])
        }
    }

    pub fn name(&self, pos: &Position) -> TypeResult<String> { self.single(pos)?.name(pos) }

    /// True iff union is (not necessarily strict) superset of other union
    pub fn is_cover(&self, other: &TypeName) -> bool {
        match (self, other) {
            (TypeName::Union { union }, TypeName::Union { union: other }) =>
                union.is_superset(other),
            _ => false
        }
    }

    pub fn substitute(
        &self,
        generics: &HashMap<String, ActualTypeName>,
        pos: &Position
    ) -> TypeResult<TypeName> {
        match self {
            TypeName::Single { ty } => Ok(TypeName::Single { ty: ty.substitute(generics, pos)? }),
            TypeName::Union { union } => {
                let union = union
                    .into_iter()
                    .map(|ty| ty.substitute(generics, pos))
                    .collect::<Result<_, _>>()?;
                Ok(TypeName::Union { union })
            }
        }
    }
}
