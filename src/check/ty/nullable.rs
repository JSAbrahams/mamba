use std::fmt;
use std::fmt::{Display, Formatter};

use crate::check::context::arg::FunctionArg;
use crate::check::result::TypeResult;
use crate::check::ty::actual::ActualType;
use crate::common::position::Position;

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

    pub fn actual_ty(&self) -> ActualType { self.actual_ty.clone() }

    pub fn constructor_args(&self, pos: &Position) -> TypeResult<Vec<FunctionArg>> {
        self.actual_ty.constructor_args(pos)
    }
}
