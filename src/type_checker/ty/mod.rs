use crate::type_checker::environment::type_name::TypeName;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub struct ExpressionType {
    pub nullable: bool,
    pub raises:   Vec<TypeName>,
    pub ty:       Option<TypeName>
}

impl ExpressionType {
    pub const ANY: ExpressionType =
        ExpressionType { nullable: false, raises: vec![], ty: None };
    pub const BOOL: ExpressionType =
        ExpressionType { nullable: false, raises: vec![], ty: Some(TypeName::BOOL) };
    pub const FLOAT: ExpressionType =
        ExpressionType { nullable: false, raises: vec![], ty: Some(TypeName::FLOAT) };
    pub const INT: ExpressionType =
        ExpressionType { nullable: false, raises: vec![], ty: Some(TypeName::INT) };
    pub const STRING: ExpressionType =
        ExpressionType { nullable: false, raises: vec![], ty: Some(TypeName::STRING) };

    pub fn new(ty: &Option<TypeName>) -> ExpressionType {
        ExpressionType { nullable: false, raises: vec![], ty: ty.clone() }
    }

    pub fn raises(self, raises: Vec<TypeName>) -> ExpressionType {
        ExpressionType { nullable: self.nullable, raises, ty: self.ty }
    }

    pub fn nullable(&self, nullable: bool) -> ExpressionType {
        ExpressionType { nullable, raises: self.raises.clone(), ty: self.ty }
    }
}
