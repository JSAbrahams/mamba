use std::fmt;
use std::fmt::{Display, Formatter};

use crate::common::position::Position;
use crate::type_checker::environment::actual_type::ActualType;
use crate::type_checker::type_result::TypeErr;
use std::collections::HashSet;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct MutableType {
    pub is_nullable: bool,
    pub is_mutable:  bool,
    pub actual_ty:   ActualType
}

impl From<&ActualType> for MutableType {
    fn from(actual_type: &ActualType) -> Self {
        MutableType { is_nullable: false, is_mutable: false, actual_ty: actual_type.clone() }
    }
}

impl Display for MutableType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let nullable = if self.is_nullable { "?" } else { "" };
        let mutable = if self.is_mutable { "mut " } else { "" };
        write!(f, "{}{}{}", mutable, self.actual_ty, nullable)
    }
}

impl MutableType {
    pub fn union(self, other: &MutableType) -> MutableType {
        MutableType {
            is_nullable: self.is_nullable || other.is_nullable,
            is_mutable:  self.is_mutable || other.is_mutable,
            actual_ty:   self.actual_ty.union(&other)
        }
    }

    pub fn mutable(self) -> MutableType {
        MutableType {
            is_nullable: self.is_nullable,
            is_mutable:  true,
            actual_ty:   self.actual_ty
        }
    }

    pub fn nullable(self) -> MutableType {
        MutableType { is_nullable: true, is_mutable: self.is_mutable, actual_ty: self.actual_ty }
    }

    pub fn field(&self, name: &str, field: &str, pos: &Position) -> Result<Option<Field>, TypeErr> {
        unimplemented!()
    }

    pub fn fun(
        &self,
        name: &str,
        args: &[MutableType],
        pos: &Position
    ) -> Result<Option<Function>, TypeErr> {
        unimplemented!()
    }
}
