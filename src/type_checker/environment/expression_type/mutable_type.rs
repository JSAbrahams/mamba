use std::fmt;
use std::fmt::{Display, Formatter};

use crate::common::position::Position;
use crate::type_checker::context::field::concrete::Field;
use crate::type_checker::context::function::concrete::Function;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::environment::expression_type::actual_type::ActualType;
use crate::type_checker::environment::expression_type::ExpressionType;
use crate::type_checker::type_result::TypeResult;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct NullableType {
    pub is_nullable: bool,
    pub actual_ty:   ActualType
}

impl Display for NullableType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let nullable = if self.is_nullable { "?" } else { "" };
        write!(f, "{}{}", self.actual_ty, nullable)
    }
}

impl From<&ActualType> for NullableType {
    fn from(actual_ty: &ActualType) -> Self {
        NullableType { is_nullable: false, actual_ty: actual_ty.clone() }
    }
}

impl NullableType {
    pub fn new(nullable: &bool, actual_ty: &ActualType) -> NullableType {
        NullableType { is_nullable: nullable.clone(), actual_ty: actual_ty.clone() }
    }

    pub fn field(&self, field: &str, pos: &Position) -> TypeResult<Field> {
        self.actual_ty.field(field, pos)
    }

    pub fn fun(
        &self,
        name: &str,
        args: &[ExpressionType],
        safe: bool,
        pos: &Position
    ) -> TypeResult<Function> {
        let args: Vec<TypeName> = args.iter().map(|arg| TypeName::from(arg)).collect();
        self.actual_ty.fun(name, &args, safe, pos)
    }
}
