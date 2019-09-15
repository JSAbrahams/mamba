use crate::common::position::Position;
use crate::type_checker::context::concrete::type_name::TypeName;
use crate::type_checker::context::concrete::Type;
use crate::type_checker::type_result::TypeErr;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub struct ExpressionType {
    pub nullable: bool,
    pub mutable:  bool,
    pub raises:   Vec<Type>,
    pub ty:       Option<Type>
}

impl Display for ExpressionType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let nullable = if self.nullable { "?" } else { "" };
        let mutable = if self.mutable { "mut " } else { "" };
        let raises = if self.raises.is_empty() { "" } else { format!(" raises [{}] ", raises) };
        let ty = if self.ty.is_some() { format!("{}", ty) } else { String::from("<unknown>") };

        write!(f, "{}{}{}{}", mutable, ty, nullable, raises)
    }
}

impl ExpressionType {
    pub fn new(ty: &Option<Type>) -> ExpressionType {
        ExpressionType { nullable: false, mutable: false, raises: vec![], ty: ty.clone() }
    }

    pub fn set_type(&self, ty: &Type, pos: &Position) -> Result<ExpressionType, TypeErr> {
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

    pub fn raises(self, raises: Vec<Type>) -> ExpressionType {
        ExpressionType { nullable: self.nullable, mutable: self.mutable, raises, ty: self.ty }
    }

    pub fn nullable(self, nullable: bool) -> ExpressionType {
        ExpressionType { nullable, mutable: self.mutable, raises: self.raises, ty: self.ty }
    }
}
