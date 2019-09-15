use std::fmt;
use std::fmt::{Display, Formatter};

use crate::common::position::Position;
use crate::type_checker::context::concrete::Type;
use crate::type_checker::environment::actual_type::ActualType;
use crate::type_checker::type_result::TypeErr;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ExpressionType {
    pub nullable:    bool,
    pub mutable:     bool,
    pub actual_type: ActualType
}

impl Display for ExpressionType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let nullable = if self.nullable { "?" } else { "" };
        let mutable = if self.mutable { "mut " } else { "" };
        write!(f, "{}{}{}", mutable, self.actual_type, nullable)
    }
}

impl ExpressionType {
    pub fn new(types: &[Type]) -> ExpressionType {
        ExpressionType {
            nullable:    false,
            mutable:     false,
            actual_type: ActualType::from(types)
        }
    }

    pub fn set_type(&self, types: &[Type], pos: &Position) -> Result<ExpressionType, TypeErr> {
        if self.actual_type == ActualType::from(types) {
            Ok(ExpressionType {
                nullable:    self.nullable,
                mutable:     self.mutable,
                actual_type: self.actual_type.clone()
            })
        } else {
            Err(TypeErr::new(pos, "Types differ"))
        }
    }

    pub fn mutable(self, mutable: bool) -> ExpressionType {
        ExpressionType { nullable: self.nullable, mutable, actual_type: self.actual_type }
    }

    pub fn nullable(self, nullable: bool) -> ExpressionType {
        ExpressionType { nullable, mutable: self.mutable, actual_type: self.actual_type }
    }
}
