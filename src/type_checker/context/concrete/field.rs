use crate::common::position::Position;
use crate::type_checker::context::generic::field::GenericField;
use crate::type_checker::context::generic::type_name::GenericType;
use crate::type_checker::type_result::TypeErr;
use std::collections::HashMap;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Field {
    pub is_py_type: bool,
    pub name:       String,
    pub ty:         Option<ActualTypeName>
}

impl Field {
    pub fn try_from(
        generic_field: &GenericField,
        generics: &HashMap<String, GenericType>,
        pos: &Position
    ) -> Result<Self, TypeErr> {
        Ok(Field {
            is_py_type: generic_field.is_py_type,
            name:       generic_field.name.clone(),
            ty:         Some(ActualTypeName::try_from(&generic_field.ty()?, generics, pos)?)
        })
    }
}
