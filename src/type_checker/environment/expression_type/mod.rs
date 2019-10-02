use std::collections::HashSet;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::iter::FromIterator;

use crate::common::position::Position;
use crate::type_checker::context::field::concrete::Field;
use crate::type_checker::context::function::concrete::Function;
use crate::type_checker::context::type_name::concrete::TypeName;
use crate::type_checker::environment::expression_type::mutable_type::MutableType;
use crate::type_checker::type_result::{TypeErr, TypeResult};

pub mod actual_type;
pub mod mutable_type;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum ExpressionType {
    Single { mut_ty: MutableType },
    Union { union: HashSet<MutableType> }
}

impl Display for ExpressionType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ExpressionType::Single { mut_ty } => write!(f, "{}", mut_ty),
            ExpressionType::Union { union } => write!(f, "{{{:#?}}}", union)
        }
    }
}

impl Into<TypeName> for ExpressionType {
    fn into(self) -> TypeName {
        match self {
            ExpressionType::Single { mut_ty } => TypeName::Single { ty: unimplemented!() },
            ExpressionType::Union { union } =>
                TypeName::Union { union: union.iter().map(|ty| unimplemented!()).collect() },
        }
    }
}

impl From<&MutableType> for ExpressionType {
    fn from(mut_ty: &MutableType) -> Self { ExpressionType::Single { mut_ty: mut_ty.clone() } }
}

impl ExpressionType {
    pub fn union(self, other: &ExpressionType) -> ExpressionType {
        match (self, other) {
            (ExpressionType::Single { mut_ty }, ExpressionType::Single { mut_ty: other }) =>
                ExpressionType::Union {
                    union: HashSet::from_iter(vec![mut_ty.clone(), other.clone()].into_iter())
                },
            (ExpressionType::Single { mut_ty }, ExpressionType::Union { union })
            | (ExpressionType::Union { union }, ExpressionType::Single { mut_ty }) =>
                ExpressionType::Union {
                    union: union
                        .union(&HashSet::from_iter(vec![mut_ty.clone()].into_iter()))
                        .collect()
                },
            (ExpressionType::Union { union }, ExpressionType::Union { union: other }) =>
                ExpressionType::Union { union: union.union(other).into_iter().collect() },
        }
    }

    pub fn single(&self, pos: &Position) -> TypeResult<MutableType> {
        match self {
            ExpressionType::Single { mut_ty } => Ok(mut_ty.clone()),
            ExpressionType::Union { .. } => Err(vec![TypeErr::new(pos, "Cannot be union")])
        }
    }

    pub fn is_mutable(&self) -> bool {
        match self {
            ExpressionType::Single { mut_ty } => mut_ty.is_mutable,
            ExpressionType::Union { union } =>
                !union.is_empty() && union.iter().all(|mut_ty| mut_ty.is_mutable),
        }
    }

    pub fn is_nullable(&self) -> bool {
        match self {
            ExpressionType::Single { mut_ty } => mut_ty.is_mutable,
            ExpressionType::Union { union } =>
                !union.is_empty() && union.iter().all(|mut_ty| mut_ty.is_nullable),
        }
    }

    pub fn field(&self, field: &str, pos: &Position) -> TypeResult<Field> {
        match &self {
            ExpressionType::Single { mut_ty } => mut_ty.field(field, pos),
            ExpressionType::Union { union } => {
                let union: Vec<Field> =
                    union.iter().map(|e_ty| e_ty.field(field, pos)).collect()?;
                let first = union.get(0);

                if union.iter().all(|e_ty| Some(e_ty) == first) {
                    Ok(first.cloned().ok_or(vec![TypeErr::new(pos, "Unknown field")])?)
                } else {
                    Err(vec![TypeErr::new(pos, "Unknown field")])
                }
            }
        }
    }

    pub fn fun(
        &self,
        name: &str,
        args: &[ExpressionType],
        safe: bool,
        pos: &Position
    ) -> TypeResult<Function> {
        match &self {
            ExpressionType::Single { mut_ty } => mut_ty.fun(name, args, safe, pos),
            ExpressionType::Union { union } => {
                let union: Vec<Function> =
                    union.iter().map(|e_ty| e_ty.fun(name, args, safe, pos)).collect()?;
                let first = union.get(0);

                if union.iter().all(|e_ty| Some(e_ty) == first) {
                    Ok(first.cloned().ok_or(vec![TypeErr::new(pos, "Unknown function")])?)
                } else {
                    Err(vec![TypeErr::new(pos, "Unknown field")])
                }
            }
        }
    }
}
