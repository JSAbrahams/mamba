use crate::common::position::Position;
use crate::parser::ast::{ASTNode, ASTNodePos};
use crate::type_checker::context::common::try_from_id;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::type_result::TypeErr;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct Field {
    pub name:    String,
    pub mutable: bool,
    position:    Position,
    ty:          Option<TypeName>
}

impl Field {
    pub fn get_type_name(&self) -> Result<TypeName, TypeErr> {
        match &self.ty {
            Some(ty) => Ok(ty.clone()),
            None => Err(TypeErr {
                position:    self.position.clone(),
                msg:         format!("{}'s type cannot be inferred", self.name),
                path:        None,
                source_line: None
            })
        }
    }

    pub fn try_from_node_pos(field: &ASTNodePos) -> Result<Field, TypeErr> {
        match &field.node {
            ASTNode::VariableDef { id_maybe_type, .. } => match &id_maybe_type.node {
                ASTNode::IdType { id, mutable, _type } => Ok(Field {
                    name:     try_from_id(id)?,
                    mutable:  *mutable,
                    position: Position::from(field),
                    ty:       match _type {
                        Some(ty) => Some(TypeName::try_from_node_pos(ty.as_ref())?),
                        None => None
                    }
                }),
                _ => Err(TypeErr::new(
                    Position::from(id_maybe_type.deref()),
                    "Expected identifier and type"
                ))
            },
            _ => Err(TypeErr::new(Position::from(field), "Expected field"))
        }
    }
}
