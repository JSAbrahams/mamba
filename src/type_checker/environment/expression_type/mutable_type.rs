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

impl From<&ActualType> for MutableType {
    fn from(actual_ty: &ActualType) -> Self {
        MutableType { is_nullable: false, is_mutable: false, actual_ty: actual_ty.clone() }
    }
}

impl MutableType {
    pub fn new(mutable: &bool, nullable: &bool, actual_ty: &ActualType) -> MutableType {
        MutableType {
            is_nullable: nullable.clone(),
            is_mutable:  mutable.clone(),
            actual_ty:   actual_ty.clone()
        }
    }

    pub fn into_mutable(self) -> Self { MutableType { is_mutable: true, ..self } }

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
