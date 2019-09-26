use std::fmt;
use std::fmt::{Display, Formatter};

use crate::common::position::Position;
use crate::type_checker::context::concrete::type_name::TypeName;
use crate::type_checker::context::concrete::Type;
use crate::type_checker::type_result::TypeErr;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum ActualType {
    Single { ty: Type },
    Tuple { types: Vec<ActualType> },
    Union { types: Vec<ActualType> },
    Fun { args: Vec<ActualType>, ret_ty: Box<ActualType> }
}

impl ActualType {
    pub fn defines_field(&self, name: &str, field: &str, pos: &Position) -> Result<bool, TypeErr> {
        match &self {
            ActualType::Single { ty } => Ok(ty.defines_field(field)),
            _ => Err(TypeErr::new(pos, "Not defined"))
        }
    }

    pub fn defines_function(
        &self,
        name: &str,
        args: &[ActualType],
        pos: &Position
    ) -> Result<bool, TypeErr> {
        match &self {
            ActualType::Single { ty } => {
                let args: Vec<_> = args.iter().map(|a| a.clone().into_type_name()).collect();
                Ok(ty.defines_function(name, &args))
            }
            _ => Err(TypeErr::new(pos, "Not defined"))
        }
    }

    pub fn union(self, other: &ActualType) -> ActualType {
        match (&self, other) {
            (ActualType::Union { types }, ActualType::Union { types: other }) =>
                ActualType::Union { types: unimplemented!() },
            (ActualType::Union { types }, other) => ActualType::Union { types: unimplemented!() },
            (this, ActualType::Union { types }) => ActualType::Union { types: unimplemented!() },
            (this, that) => ActualType::Union { types: vec![this.clone(), that.clone()] }
        }
    }

    fn into_type_name(self) -> TypeName {
        match self {
            ActualType::Single { ty } => ty.name,
            ActualType::Tuple { types } => TypeName::Tuple {
                ty_names: types.iter().map(|at| at.clone().into_type_name()).collect()
            },
            ActualType::Union { types } => TypeName::Union {
                ty_names: types.iter().map(|at| at.clone().into_type_name()).collect()
            },
            ActualType::Fun { args, ret_ty } => {
                let ret_ty: TypeName = ret_ty.clone().into_type_name();
                TypeName::Fun {
                    args:   args.iter().map(|a| a.clone().into_type_name()).collect(),
                    ret_ty: Box::from(ret_ty)
                }
            }
        }
    }
}

impl Display for ActualType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match &self {
            ActualType::Single { ty } => write!(f, "{}", ty),
            ActualType::Tuple { types } => write!(f, "({:#?})", types),
            ActualType::Union { types } => write!(f, "{{{:#?}}}", types),
            ActualType::Fun { args, ret_ty } => write!(f, "({:#?}) -> {}", args, ret_ty)
        }
    }
}
