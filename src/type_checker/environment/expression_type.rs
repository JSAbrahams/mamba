use crate::common::position::Position;
use crate::type_checker::context::concrete::Type;
use crate::type_checker::type_result::TypeErr;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ExpressionType {
    pub nullable: bool,
    pub mutable:  bool,
    pub types:    Vec<Type>
}

impl Display for ExpressionType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let nullable = if self.nullable { "?" } else { "" };
        let mutable = if self.mutable { "mut " } else { "" };
        let ty = if self.types.is_empty() {
            String::from("<unknown>")
        } else {
            format!("({:#?})", self.types)
        };

        write!(f, "{}{}{}", mutable, ty, nullable)
    }
}

impl ExpressionType {
    pub fn new(types: &[Type]) -> ExpressionType {
        ExpressionType { nullable: false, mutable: false, types: Vec::from(types) }
    }

    pub fn set_type(&self, types: &[Type], pos: &Position) -> Result<ExpressionType, TypeErr> {
        if self.types.as_slice() == types {
            Ok(ExpressionType {
                nullable: self.nullable,
                mutable:  self.mutable,
                types:    self.types.clone()
            })
        } else {
            Err(TypeErr::new(pos, "Types differ"))
        }
    }

    pub fn mutable(self, mutable: bool) -> ExpressionType {
        ExpressionType { nullable: self.nullable, mutable, types: self.types }
    }

    pub fn nullable(self, nullable: bool) -> ExpressionType {
        ExpressionType { nullable, mutable: self.mutable, types: self.types }
    }
}
