use std::fmt;
use std::fmt::{Display, Formatter};

use crate::common::position::Position;
use crate::type_checker::context::concrete::Type;
use crate::type_checker::type_result::TypeErr;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum ActualType {
    Single { ty: Type },
    Tuple { types: Vec<Type> },
    Function { args: Vec<Type>, ret_ty: Box<ActualType> }
}

impl ActualType {
    pub fn ty(&self, pos: &Position) -> Result<Type, TypeErr> {
        match self {
            ActualType::Single { ty } => Ok(ty.clone()),
            ActualType::Tuple { .. } => Err(TypeErr::new(pos, "Tuples don't have direct type")),
            ActualType::Function { .. } =>
                Err(TypeErr::new(pos, "Functions don't have direct type")),
        }
    }
}

impl From<&[Type]> for ActualType {
    /// Create an actual type of an expression, which may either be a single
    /// type, or a tuple of types.
    fn from(types: &[Type]) -> Self {
        if types.len() == 1 {
            ActualType::Single { ty: types.get(0).unwrap_or_else(|| unreachable!()).clone() }
        } else {
            ActualType::Tuple { types: types.to_vec() }
        }
    }
}

impl Display for ActualType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match &self {
            ActualType::Single { ty } => write!(f, "{}", ty),
            ActualType::Tuple { types } => write!(f, "({:#?})", types),
            ActualType::Function { args, ret_ty } => write!(f, "({:#?}) -> {}", args, ret_ty)
        }
    }
}
