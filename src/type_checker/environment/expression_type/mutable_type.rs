use std::fmt;
use std::fmt::{Display, Formatter};

use crate::common::position::Position;
use crate::type_checker::context::concrete::field::Field;
use crate::type_checker::context::concrete::function::Function;
use crate::type_checker::context::concrete::Type;
use crate::type_checker::environment::expression_type::actual_type::ActualType;
use crate::type_checker::type_result::TypeResult;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct MutableType {
    pub is_nullable: bool,
    pub is_mutable:  bool,
    pub actual_ty:   ActualType
}

impl Display for MutableType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let nullable = if self.is_nullable { "?" } else { "" };
        let mutable = if self.is_mutable { "mut " } else { "" };
        write!(f, "{}{}{}", mutable, self.actual_ty, nullable)
    }
}

impl From<&Type> for MutableType {
    fn from(ty: &Type) -> Self { MutableType::from(&ActualType::from(ty)) }
}

impl From<&ActualType> for MutableType {
    fn from(actual_ty: &ActualType) -> Self {
        MutableType { is_nullable: false, is_mutable: false, actual_ty: actual_ty.clone() }
    }
}

impl MutableType {
    pub fn field(&self, field: &str, pos: &Position) -> TypeResult<Field> {
        self.actual_ty.field(field, pos)
    }

    pub fn fun(&self, name: &str, args: &[MutableType], pos: &Position) -> TypeResult<Function> {
        self.actual_ty.fun(name, args.iter().map(|a| a.actual_ty).collect(), pos)
    }
}
