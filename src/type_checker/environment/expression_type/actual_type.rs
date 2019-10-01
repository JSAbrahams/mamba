use std::fmt;
use std::fmt::{Display, Formatter};

use crate::common::position::Position;
use crate::type_checker::context::concrete::field::Field;
use crate::type_checker::context::concrete::function::Function;
use crate::type_checker::context::concrete::Type;
use crate::type_checker::environment::expression_type::ExpressionType;
use crate::type_checker::type_result::{TypeErr, TypeResult};

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum ActualType {
    Single { ty: TypeVariant },
    Tuple { types: Vec<ExpressionType> },
    AnonFun { args: Vec<ExpressionType>, ret_ty: ExpressionType }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum TypeVariant {
    Ty(Type),
    Fun(Function),
    F(Field)
}

impl Display for TypeVariant {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            TypeVariant::Ty(t) => write!(f, "{}", t),
            TypeVariant::Fun(f) => write!(f, "{}", f),
            TypeVariant::F(f) => write!(f, "{}", f)
        }
    }
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
    fn from(ty: &Type) -> Self { ActualType::Single { ty: TypeVariant::Ty(ty.clone()) } }
}

impl ActualType {
    pub fn field(&self, field: &str, pos: &Position) -> TypeResult<Field> {
        match &self {
            ActualType::Single { ty } =>
                Ok(ty.field(field).ok_or(vec![TypeErr::new(pos, "Undefined field")])?),
            _ => Err(vec![TypeErr::new(pos, "Undefined field")])
        }
    }

    pub fn fun(&self, name: &str, args: &[ActualType], pos: &Position) -> TypeResult<Function> {
        match &self {
            ActualType::Single { ty } => {
                let args: Vec<_> = args.iter().map(|a| a.clone().into_type_name()).collect();
                Ok(ty.function(name, &args).ok_or(vec![TypeErr::new(pos, "Undefined function")])?)
            }
            _ => Err(vec![TypeErr::new(pos, "Undefined function")])
        }
    }

    fn into_type_name(self) -> ActualTypeName {
        match self {
            ActualType::Single { ty } => ty.name,
            ActualType::Tuple { types } => ActualTypeName::Tuple {
                ty_names: types.iter().map(|at| at.clone().into_type_name()).collect()
            },
            ActualType::AnonFun { args, ret_ty } => {
                let ret_ty: ActualTypeName = ret_ty.clone().into_type_name();
                ActualTypeName::AnonFun {
                    args:   args.iter().map(|a| a.clone().into_type_name()).collect(),
                    ret_ty: Box::from(ret_ty)
                }
            }
        }
    }
}
