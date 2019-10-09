use std::fmt;
use std::fmt::{Display, Formatter};

use crate::common::position::Position;
use crate::type_checker::context::field::concrete::Field;
use crate::type_checker::context::function::concrete::Function;
use crate::type_checker::context::ty::concrete::Type;
use crate::type_checker::context::type_name::actual::ActualTypeName;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::environment::expression_type::ExpressionType;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use std::ops::Deref;

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
            ActualType::Tuple { types } => write!(f, "({:#?})", types),
            ActualType::AnonFun { args, ret_ty } => write!(f, "({:#?}) -> {}", args, ret_ty)
        }
    }
}

impl From<&Type> for ActualType {
    fn from(ty: &Type) -> Self { ActualType::Single { ty: ty.clone() } }
}

impl ActualType {
    pub fn name(&self) -> ActualTypeName {
        match self {
            ActualType::Single { ty } => ty.name.clone(),
            ActualType::Tuple { types } =>
                ActualTypeName::Tuple { ty_names: types.iter().map(|ty| ty.name()).collect() },
            ActualType::AnonFun { args, ret_ty } => ActualTypeName::AnonFun {
                args:   args.iter().map(|ty| ty.name()).collect(),
                ret_ty: Box::new(ret_ty.deref().name())
            }
        }
    }

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
            ActualType::Single { ty } => ty.fun(name, &args, safe, pos),
            _ => Err(vec![TypeErr::new(pos, "Undefined function")])
        }
    }
}
