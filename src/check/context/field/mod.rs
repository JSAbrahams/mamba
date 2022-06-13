use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use std::fmt;

use crate::check::context::field::generic::GenericField;
use crate::check::context::LookupField;
use crate::check::name::{Empty, Name, Substitute};
use crate::check::name::string_name::StringName;
use crate::check::result::TypeErr;
use crate::common::position::Position;
use crate::Context;

pub mod union;
pub mod generic;
pub mod python;

/// A Field, which may either be top-level, or optionally within a class.
///
/// May have a type.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Field {
    pub is_py_type: bool,
    pub name: String,
    pub mutable: bool,
    pub in_class: Option<StringName>,
    pub ty: Name,
    pub assigned_to: bool,
}

impl Display for Field {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let ty = if self.ty.is_empty() { String::new() } else { format!(": {}", self.ty) };
        write!(f, "{}{}", &self.name, ty)
    }
}

impl LookupField<&str, Field> for Context {
    /// Look up a field and substitutes generics to yield a Field.
    fn field(&self, field: &str, pos: Position) -> Result<Field, Vec<TypeErr>> {
        if let Some(generic_field) = self.fields.iter().find(|c| c.name == field) {
            let generics = HashMap::new();
            Field::try_from((generic_field, &generics, pos))
        } else {
            let msg = format!("Field {} is undefined.", field);
            Err(vec![TypeErr::new(pos, &msg)])
        }
    }
}

impl TryFrom<(&GenericField, &HashMap<Name, Name>, Position)> for Field {
    type Error = Vec<TypeErr>;

    fn try_from(
        (field, generics, pos): (&GenericField, &HashMap<Name, Name>, Position)
    ) -> Result<Self, Self::Error> {
        Ok(Field {
            is_py_type: field.is_py_type,
            name: field.name.clone(),
            mutable: field.mutable,
            in_class: match &field.in_class {
                Some(in_class) => Some(in_class.substitute(generics, pos)?),
                None => None
            },
            ty: match &field.ty {
                Some(ty) => ty.substitute(generics, pos)?,
                None => Name::empty()
            },
            assigned_to: field.assigned_to,
        })
    }
}
