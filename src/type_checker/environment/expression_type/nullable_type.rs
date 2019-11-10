use std::fmt;
use std::fmt::{Display, Formatter};

use crate::common::position::Position;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::environment::expression_type::actual_type::ActualType;
use crate::type_checker::type_result::TypeResult;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct NullableType {
    pub is_nullable: bool,
    actual_ty:       ActualType
}

impl Display for NullableType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let nullable = if self.is_nullable { "?" } else { "" };
        write!(f, "{}{}", self.actual_ty, nullable)
    }
}

impl NullableType {
    pub fn new(nullable: bool, actual_ty: &ActualType) -> NullableType {
        NullableType { is_nullable: nullable.clone(), actual_ty: actual_ty.clone() }
    }

    pub fn as_nullable(&self) -> NullableType {
        NullableType { is_nullable: true, actual_ty: self.actual_ty.clone() }
    }

    pub fn actual_ty(&self) -> ActualType { self.actual_ty.clone() }

    pub fn constructor(&self, args: &[TypeName], pos: &Position) -> TypeResult<NullableType> {
        let actual_ty = self.actual_ty.args(args, pos)?;
        Ok(NullableType { is_nullable: self.is_nullable, actual_ty })
    }
}
