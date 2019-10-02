use std::fmt;
use std::fmt::{Display, Formatter};

use crate::common::position::Position;
use crate::type_checker::context::field::concrete::Field;
use crate::type_checker::context::function::concrete::Function;
use crate::type_checker::context::ty::concrete::Type;
use crate::type_checker::context::type_name::concrete::TypeName;
use crate::type_checker::environment::expression_type::mutable_type::MutableType;
use crate::type_checker::type_result::{TypeErr, TypeResult};

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum ActualType {
    Single { ty: Type },
    Tuple { types: Vec<MutableType> },
    AnonFun { args: Vec<MutableType>, ret_ty: Box<MutableType> }
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
    fn from(ty: &Type) -> Self { ActualType::Single { ty: ty.clone() } }
}

impl ActualType {
    pub fn field(&self, field: &str, pos: &Position) -> TypeResult<Field> {
        match &self {
            ActualType::Single { ty } =>
                Ok(ty.field(field).ok_or(vec![TypeErr::new(pos, "Undefined field")])?),
            _ => Err(vec![TypeErr::new(pos, "Undefined field")])
        }
    }

    pub fn fun(
        &self,
        name: &str,
        args: &[TypeName],
        safe: bool,
        pos: &Position
    ) -> TypeResult<Function> {
        match &self {
            ActualType::Single { ty } =>
                Ok(ty.function(name, &args).ok_or(vec![TypeErr::new(pos, "Undefined function")])?),
            _ => Err(vec![TypeErr::new(pos, "Undefined function")])
        }
    }
}
