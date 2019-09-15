use std::fmt;
use std::fmt::{Display, Formatter};

use crate::type_checker::context::concrete::Type;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum ActualType {
    Single { ty: Type },
    Tuple { types: Vec<Type> },
    Function { args: Vec<Type>, ret_ty: Box<ActualType> }
}

impl From<&[Type]> for ActualType {
    fn from(types: &[Type]) -> Self {
        if types.len() == 1 {
            ActualType::Single { ty: types.get(0).unwrap_or_else(|| unreachable!()).clone() }
        } else {
            ActualType::Tuple { types: vec![] }
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
