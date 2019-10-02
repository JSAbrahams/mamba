use crate::common::position::Position;
use crate::type_checker::context::type_name::concrete::actual::ActualTypeName;
use crate::type_checker::context::type_name::generic::GenericTypeName;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use core::fmt;
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};

pub mod actual;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TypeName {
    Single { ty: ActualTypeName },
    Union { union: HashSet<ActualTypeName> }
}

impl Display for TypeName {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            TypeName::Single { ty } => write!(f, "{}", ty),
            TypeName::Union { union } => write!(f, "{{{:#?}}}", union)
        }
    }
}

impl TryFrom<(&GenericTypeName, &Position)> for TypeName {
    type Error = Vec<TypeErr>;

    fn try_from((type_name, pos): (&GenericTypeName, &Position)) -> Result<Self, Self::Error> {
        match type_name {
            GenericTypeName::Single { ty } =>
                Ok(TypeName::Single { ty: ActualTypeName::try_from((ty, &HashMap::new(), pos))? }),
            GenericTypeName::Union { union } => {
                let (union, errs): (Vec<_>, Vec<_>) = union
                    .iter()
                    .map(|ty| ActualTypeName::try_from((ty, &HashMap::new(), pos)))
                    .partition(Result::is_ok);

                if errs.is_empty() {
                    Ok(TypeName::Union { union: union.into_iter().map(Result::unwrap).collect() })
                } else {
                    Err(errs.into_iter().map(Result::unwrap_err).flatten().collect())
                }
            }
        }
    }
}

impl TryFrom<(&GenericTypeName, &HashMap<String, ActualTypeName>, &Position)> for TypeName {
    type Error = Vec<TypeErr>;

    fn try_from(
        (gen_type_name, generics, pos): (
            &GenericTypeName,
            &HashMap<String, ActualTypeName>,
            &Position
        )
    ) -> TypeResult<Self> {
        match gen_type_name {
            GenericTypeName::Single { ty } =>
                Ok(TypeName::Single { ty: ActualTypeName::try_from((ty, generics, pos))? }),
            GenericTypeName::Union { union } => {
                let (union, errs): (Vec<_>, Vec<_>) = union
                    .iter()
                    .map(|ty| ActualTypeName::try_from((ty, generics, pos)))
                    .partition(Result::is_ok);

                if errs.is_empty() {
                    Ok(TypeName::Union { union: union.into_iter().map(Result::unwrap).collect() })
                } else {
                    Err(errs.into_iter().map(Result::unwrap_err).flatten().collect())
                }
            }
        }
    }
}

impl TypeName {
    pub fn new(lit: &str, generics: &[ActualTypeName]) -> TypeName {
        TypeName::Single {
            ty: ActualTypeName::Single {
                lit:      String::from(lit),
                generics: Vec::from(generics)
            }
        }
    }

    pub fn single(&self, pos: &Position) -> TypeResult<ActualTypeName> {
        match self {
            TypeName::Single { ty } => Ok(ty.clone()),
            TypeName::Union { .. } => Err(vec![TypeErr::new(pos, "Unions not supported here")])
        }
    }

    pub fn name(&self, pos: &Position) -> TypeResult<String> { self.single(pos)?.name(pos) }

    /// True iff union is (not necessarily strict) superset of other union
    pub fn is_cover(&self, _: &TypeName) -> bool { unimplemented!() }
}
