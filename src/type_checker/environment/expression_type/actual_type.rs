use std::fmt;
use std::fmt::{Display, Formatter};

use crate::common::position::Position;
use crate::type_checker::context::concrete::field::Field;
use crate::type_checker::context::concrete::function::Function;
use crate::type_checker::context::concrete::type_name::TypeName;
use crate::type_checker::context::concrete::Type;
use crate::type_checker::environment::expression_type::actual_type::TypeVariant::{Fld, Fun, Ty};
use crate::type_checker::environment::expression_type::ExpressionType;
use crate::type_checker::type_result::{TypeErr, TypeResult};

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum ActualType {
    Single { ty: TypeVariant },
    Tuple { types: Vec<ExpressionType> },
    AnonFun { args: Vec<ExpressionType>, ret_ty: Box<ExpressionType> }
}

pub enum TypeVariant {
    Ty(Type),
    Fun(Function),
    Fld(Field)
}

impl Display for ActualType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match &self {
            ActualType::Single { ty } => write!(f, "{}", ty),
            ActualType::Tuple { types } => write!(f, "({:#?})", types),
            ActualType::AnonFun { args, ret_ty } => write!(f, "({:#?}) -> {}", args, ret_ty)
        }
    }
}

impl From<&Type> for ActualType {
    fn from(ty: &Type) -> Self { ActualType::Single { ty: Ty(ty.clone()) } }
}

impl From<&Function> for ActualType {
    fn from(fun: &Function) -> Self { ActualType::Single { ty: Fun(fun.clone()) } }
}

impl From<&Field> for ActualType {
    fn from(field: &Field) -> Self { ActualType::Single { ty: Fld(field.clone()) } }
}

impl ActualType {
    pub fn field(&self, field: &str, pos: &Position) -> TypeResult<Field> {
        match &self {
            ActualType::Single { ty } =>
                Ok(ty.field(field).ok_or(vec![TypeErr::new(pos, "Undefined field")])?),
            _ => Err(vec![TypeErr::new(pos, "Undefined field")])
        }
    }

    pub fn fun(&self, name: &str, args: &[TypeName], pos: &Position) -> TypeResult<Function> {
        match &self {
            ActualType::Single { ty } =>
                Ok(ty.function(name, &args).ok_or(vec![TypeErr::new(pos, "Undefined function")])?),
            _ => Err(vec![TypeErr::new(pos, "Undefined function")])
        }
    }
}
