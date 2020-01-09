use std::fmt;
use std::fmt::{Display, Formatter};

use crate::common::position::Position;
use crate::type_checker::context::field::concrete::Field;
use crate::type_checker::context::function::concrete::Function;
use crate::type_checker::context::function_arg::concrete::FunctionArg;
use crate::type_checker::infer_type::actual::ActualType;
use crate::type_checker::infer_type::expression::ExpressionType;
use crate::type_checker::type_name::TypeName;
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

    pub fn anon_fun(&self, args: &[TypeName], pos: &Position) -> TypeResult<ExpressionType> {
        self.actual_ty.anon_fun(args, pos)
    }

    pub fn fun(&self, name: &str, args: &[TypeName], pos: &Position) -> TypeResult<Function> {
        self.actual_ty.fun(name, args, pos)
    }

    pub fn constructor(&self, args: &[TypeName], pos: &Position) -> TypeResult<NullableType> {
        let actual_ty = self.actual_ty.constructor(args, pos)?;
        Ok(NullableType { is_nullable: self.is_nullable, actual_ty })
    }

    pub fn constructor_args(&self, pos: &Position) -> TypeResult<Vec<FunctionArg>> {
        self.actual_ty.constructor_args(pos)
    }
}
