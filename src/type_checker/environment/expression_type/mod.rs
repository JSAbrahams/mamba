use crate::common::position::Position;
use crate::type_checker::context::field::concrete::Field;
use crate::type_checker::context::function::concrete::Function;
use crate::type_checker::environment::expression_type::mutable_type::MutableType;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use std::collections::HashSet;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::iter::FromIterator;

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

impl From<&MutableType> for ExpressionType {
    fn from(mut_ty: &MutableType) -> Self { ExpressionType::Single { mut_ty: mut_ty.clone() } }
}

impl ExpressionType {
    pub fn union(self, other: &ExpressionType) -> ExpressionType {
        match (self, other) {
            (ExpressionType::Single { mut_ty }, ExpressionType::Single { mut_ty: other }) =>
                ExpressionType::Union { union: HashSet::from_iter(vec![mut_ty, other].iter()) },
            (ExpressionType::Single { mut_ty }, ExpressionType::Union { union })
            | (ExpressionType::Union { union }, ExpressionType::Single { mut_ty }) => {
                let mut union = union.clone();
                union.insert(mut_ty);
                ExpressionType::Union { union }
            }
            (ExpressionType::Union { union }, ExpressionType::Union { union: other }) => {
                let mut union = union.clone();
                union.extend(other);
                ExpressionType::Union { union }
            }
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
                let union = union.iter().map(|e_ty| e_ty.field(field, pos)).collect()?;
                let first = union.get(0);
                if union.iter.all(|e_ty| e_ty == first) {
                    Ok(first.ok_or(vec![TypeErr::new(pos, "Unknown field")])?)
                } else {
                    Err(vec![TypeErr::new(pos, "Unknown field")])
                }
            }
        }
    }

    pub fn fun(&self, name: &str, args: &[ExpressionType], pos: &Position) -> TypeResult<Function> {
        match &self {
            ExpressionType::Single { mut_ty } => mut_ty.function(name, args, pos),
            ExpressionType::Union { union } => {
                let union = union.iter().map(|e_ty| e_ty.fun(name, args, pos)).collect()?;
                let first = union.get(0);
                if union.iter.all(|e_ty| e_ty == first) {
                    Ok(first.ok_or(vec![TypeErr::new(pos, "Unknown field")])?)
                } else {
                    Err(vec![TypeErr::new(pos, "Unknown field")])
                }
            }
        }
    }
}
