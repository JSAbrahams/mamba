use std::ops::Deref;

use crate::common::position::Position;
use crate::parser::ast::{ASTNode, ASTNodePos};
use crate::type_checker::context::common::try_from_id;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::context::ReturnType;
use crate::type_checker::type_result::TypeErr;

#[derive(Debug, Clone)]
pub struct Field {
    pub name:     String,
    pub mutable:  bool,
    pub position: Position,
    ty:           Option<TypeName>
}

impl Field {
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

impl ReturnType for Field {
    fn with_return_type_name(self, ty: TypeName) -> Result<Self, TypeErr> {
        if self.ty.is_some() && self.ty.unwrap() != ty {
            Err(TypeErr::new(self.position, "Inferred type not equal to signature"))
        } else {
            Ok(Field {
                name:     self.name,
                mutable:  self.mutable,
                position: self.position.clone(),
                ty:       Some(ty)
            })
        }
    }

    fn get_return_type_name(&self) -> Result<TypeName, TypeErr> {
        match &self.ty {
            Some(ty) => Ok(ty.clone()),
            None => Err(TypeErr::new(self.position.clone(), "Type cannot be inferred"))
        }
    }
}
