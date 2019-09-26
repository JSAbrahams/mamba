use std::fmt;
use std::fmt::{Display, Formatter};

use crate::type_checker::environment::actual_type::ActualType;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ExpressionType {
    pub is_nullable: bool,
    pub is_mutable:  bool,
    pub actual_ty:   ActualType
}

impl From<&ActualType> for ExpressionType {
    fn from(actual_type: &ActualType) -> Self {
        ExpressionType { is_nullable: false, is_mutable: false, actual_ty: actual_type.clone() }
    }
}

impl Display for ExpressionType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let nullable = if self.is_nullable { "?" } else { "" };
        let mutable = if self.is_mutable { "mut " } else { "" };
        write!(f, "{}{}{}", mutable, self.actual_ty, nullable)
    }
}

impl ExpressionType {
    pub fn union(self, other: &ExpressionType) -> ExpressionType {
        ExpressionType {
            is_nullable: self.is_nullable || other.is_nullable,
            is_mutable:  self.is_mutable || other.is_mutable,
            actual_ty:   self.actual_ty.union(&other.actual_ty)
        }
    }

    pub fn mutable(self) -> ExpressionType {
        ExpressionType {
            is_nullable: self.is_nullable,
            is_mutable:  true,
            actual_ty:   self.actual_ty
        }
    }

    pub fn nullable(self) -> ExpressionType {
        ExpressionType {
            is_nullable: true,
            is_mutable:  self.is_mutable,
            actual_ty:   self.actual_ty
        }
    }
}
