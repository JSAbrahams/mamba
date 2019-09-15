use std::fmt;
use std::fmt::{Display, Formatter};

use crate::common::position::Position;
use crate::type_checker::environment::type_name::TypeName;
use crate::type_checker::type_result::TypeErr;

#[derive(Clone)]
pub struct ExpressionType {
    pub nullable: bool,
    pub mutable:  bool,
    pub raises:   Vec<TypeName>,
    pub ty:       Option<TypeName>
}

impl ExpressionType {
    pub fn new(ty: &Option<TypeName>) -> ExpressionType {
        ExpressionType { nullable: false, mutable: false, raises: vec![], ty: ty.clone() }
    }

    pub fn set_type(&self, ty: &TypeName, pos: &Position) -> Result<ExpressionType, TypeErr> {
        if self.ty.map_or(self.ty.is_none(), |self_ty| self_ty == ty) {
            Ok(ExpressionType {
                nullable: self.nullable,
                mutable:  self.mutable,
                raises:   self.raises.clone(),
                ty:       Some(ty.clone())
            })
        } else {
            Err(TypeErr::new(pos, "Types differ"))
        }
    }

    pub fn mutable(self, mutable: bool) -> ExpressionType {
        ExpressionType { nullable: self.nullable, mutable, raises: self.raises, ty: self.ty }
    }

    pub fn raises(self, raises: Vec<TypeName>) -> ExpressionType {
        ExpressionType { nullable: self.nullable, mutable: self.mutable, raises, ty: self.ty }
    }

    pub fn nullable(self, nullable: bool) -> ExpressionType {
        ExpressionType { nullable, mutable: self.mutable, raises: self.raises, ty: self.ty }
    }
}
