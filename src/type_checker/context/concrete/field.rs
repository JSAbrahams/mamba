use crate::common::position::Position;
use crate::type_checker::context::concrete::type_name::TypeName;
use crate::type_checker::context::generic::field::GenericField;
use crate::type_checker::context::generic::type_name::GenericTypeName;
use crate::type_checker::type_result::TypeErr;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty:   Option<TypeName>
}

impl Field {
    pub fn try_from(
        generic_field: &GenericField,
        generics: &HashMap<String, GenericTypeName>,
        pos: &Position
    ) -> Result<Self, TypeErr> {
        Ok(Field {
            name: generic_field.name.clone(),
            ty:   Some(TypeName::try_from(&generic_field.ty()?, generics, pos)?)
        })
    }
}
