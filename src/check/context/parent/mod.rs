use std::collections::HashMap;
use std::convert::TryFrom;

use crate::check::context::parent::generic::GenericParent;
use crate::check::name::true_name::TrueName;
use crate::check::name::{Name, Substitute};
use crate::check::result::TypeErr;
use crate::common::position::Position;

pub mod generic;
pub mod python;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Parent {
    pub is_py_type: bool,
    pub name: TrueName,
    pub pos: Position,
}

impl TryFrom<(&GenericParent, &HashMap<Name, Name>, Position)> for TrueName {
    type Error = Vec<TypeErr>;

    fn try_from(
        (parent, generics, pos): (&GenericParent, &HashMap<Name, Name>, Position),
    ) -> Result<Self, Self::Error> {
        parent.name.substitute(generics, pos)
    }
}
