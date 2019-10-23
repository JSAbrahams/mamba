use std::collections::HashSet;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;

use crate::common::position::Position;
use crate::type_checker::context::field::concrete::Field;
use crate::type_checker::context::function::concrete::Function;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::environment::expression_type::nullable_type::NullableType;
use crate::type_checker::type_result::{TypeErr, TypeResult};

pub mod actual_type;
pub mod nullable_type;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum ExpressionType {
    Single { ty: NullableType },
    Union { union: HashSet<NullableType> }
}

impl Hash for ExpressionType {
    /// Hash ExpressionType
    ///
    /// Due to ExpressionTypes containing HashSets, the runtime is O(n) instead
    /// of O(1).
    fn hash<H: Hasher>(&self, state: &mut H) {
        match &self {
            ExpressionType::Single { ty } => ty.hash(state),
            ExpressionType::Union { union } => union.iter().for_each(|ty| ty.hash(state))
        }
    }
}

impl From<&NullableType> for ExpressionType {
    fn from(nullable_type: &NullableType) -> Self {
        ExpressionType::Single { ty: nullable_type.clone() }
    }
}

impl Display for ExpressionType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ExpressionType::Single { ty } => write!(f, "{}", ty),
            ExpressionType::Union { union } => write!(
                f,
                "{{{}}}",
                {
                    let mut string = String::new();
                    union.iter().for_each(|e_ty| string.push_str(&format!("{}, ", e_ty)));
                    string.remove(string.len() - 2);
                    string
                }
                .trim_end()
            )
        }
    }
}

impl ExpressionType {
    pub fn union(self, other: &ExpressionType) -> ExpressionType {
        match (&self, other) {
            (ExpressionType::Single { ty: mut_ty }, ExpressionType::Single { ty: other }) =>
                ExpressionType::Union {
                    union: HashSet::from_iter(vec![mut_ty.clone(), other.clone()].into_iter())
                },
            (ExpressionType::Single { ty: mut_ty }, ExpressionType::Union { union })
            | (ExpressionType::Union { union }, ExpressionType::Single { ty: mut_ty }) =>
                ExpressionType::Union {
                    union: union
                        .union(&HashSet::from_iter(vec![mut_ty.clone()].into_iter()))
                        .cloned()
                        .collect()
                },
            (ExpressionType::Union { union }, ExpressionType::Union { union: other }) =>
                ExpressionType::Union { union: union.union(other).cloned().collect() },
        }
    }

    pub fn single(&self, pos: &Position) -> TypeResult<NullableType> {
        match self {
            ExpressionType::Single { ty: mut_ty } => Ok(mut_ty.clone()),
            ExpressionType::Union { .. } => Err(vec![TypeErr::new(pos, "Cannot be union")])
        }
    }

    pub fn is_nullable(&self) -> bool {
        match self {
            ExpressionType::Single { ty: mut_ty } => mut_ty.is_nullable,
            ExpressionType::Union { union } =>
                !union.is_empty() && union.iter().all(|mut_ty| mut_ty.is_nullable),
        }
    }

    pub fn field(&self, field: &str, nullable: bool, pos: &Position) -> TypeResult<Field> {
        match &self {
            ExpressionType::Single { ty: mut_ty } =>
                mut_ty.actual_ty_safe(nullable, pos)?.field(field, pos),
            ExpressionType::Union { union } => {
                let union: Vec<Field> = union
                    .iter()
                    .map(|e_ty| e_ty.actual_ty_safe(nullable, pos)?.field(field, pos))
                    .collect::<Result<_, _>>()?;
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
        args: &[TypeName],
        nullable: bool,
        pos: &Position
    ) -> TypeResult<Function> {
        match &self {
            ExpressionType::Single { ty: mut_ty } =>
                mut_ty.actual_ty_safe(nullable, pos)?.fun(name, args, pos),
            ExpressionType::Union { union } => {
                let union: Vec<Function> = union
                    .iter()
                    .map(|e_ty| e_ty.actual_ty_safe(nullable, pos)?.fun(name, args, pos))
                    .collect::<Result<_, Vec<TypeErr>>>()?;
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
