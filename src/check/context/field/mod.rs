use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};

use crate::check::context::field::generic::GenericField;
use crate::check::context::name::{DirectName, Name, NameUnion};
use crate::check::result::TypeErr;
use crate::common::position::Position;
use std::fmt;

pub mod generic;
pub mod python;

/// A Field, which may either be top-level, or optionally within a class.
///
/// May have a type.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Field {
    pub is_py_type: bool,
    pub name:       String,
    pub private:    bool,
    pub mutable:    bool,
    pub in_class:   Option<DirectName>,
    pub ty:         NameUnion
}

impl Display for Field {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let ty = if self.ty.is_empty() { String::new() } else { format!(": {}", self.ty) };
        write!(f, "{}{}", &self.name, ty)
    }
}

impl TryFrom<(&GenericField, &HashMap<String, Name>, &Position)> for Field {
    type Error = Vec<TypeErr>;

    fn try_from(
        (field, generics, pos): (&GenericField, &HashMap<String, Name>, &Position)
    ) -> Result<Self, Self::Error> {
        Ok(Field {
            is_py_type: field.is_py_type,
            name:       field.name.clone(),
            private:    field.private,
            mutable:    field.mutable,
            in_class:   match &field.in_class {
                Some(in_class) => Some(in_class.substitute(generics, pos)?),
                None => None
            },
            ty:         match &field.ty {
                Some(ty) => ty.substitute(generics, pos)?,
                None => NameUnion::empty()
            }
        })
    }
}
