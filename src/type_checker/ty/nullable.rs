use std::fmt;
use std::fmt::{Display, Formatter};

use crate::common::position::Position;
use crate::type_checker::checker_result::TypeResult;
use crate::type_checker::context::field::concrete::Field;
use crate::type_checker::context::function_arg::concrete::FunctionArg;
use crate::type_checker::ty::actual::ActualType;

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
    pub fn new(is_nullable: bool, actual_ty: &ActualType) -> NullableType {
        NullableType { is_nullable, actual_ty: actual_ty.clone() }
    }

    pub fn as_nullable(&self) -> NullableType {
        NullableType { is_nullable: true, actual_ty: self.actual_ty.clone() }
    }

    pub fn actual_ty(&self) -> ActualType { self.actual_ty.clone() }

    pub fn field(&self, field: &str, pos: &Position) -> TypeResult<Field> {
        self.actual_ty.field(field, pos)
    }

    pub fn constructor_args(&self, pos: &Position) -> TypeResult<Vec<FunctionArg>> {
        self.actual_ty.constructor_args(pos)
    }
}
