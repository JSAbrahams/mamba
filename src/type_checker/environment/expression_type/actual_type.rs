use std::fmt;
use std::fmt::{Display, Formatter};

use crate::common::position::Position;
use crate::type_checker::context::field::concrete::Field;
use crate::type_checker::context::function::concrete::Function;
use crate::type_checker::context::ty::concrete::Type;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::environment::expression_type::ExpressionType;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use crate::type_checker::util::comma_delimited;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum ActualType {
    Single { ty: Type },
    Tuple { types: Vec<ExpressionType> },
    AnonFun { args: Vec<ExpressionType>, ret_ty: Box<ExpressionType> }
}

impl Display for ActualType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match &self {
            ActualType::Single { ty } => write!(f, "{}", ty),
            ActualType::Tuple { types } => write!(f, "({})", comma_delimited(types)),
            ActualType::AnonFun { args, ret_ty } =>
                write!(f, "({}) -> {}", comma_delimited(args), ret_ty),
        }
    }
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
            ActualType::Single { ty } => ty.fun(name, args, pos),
            _ => Err(vec![TypeErr::new(pos, "Undefined function")])
        }
    }
}
