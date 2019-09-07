use crate::common::position::Position;
use crate::parser::ast::{ASTNode, ASTNodePos};
use crate::type_checker::type_result::TypeErr;
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeName {
    Single { lit: String, generics: Vec<TypeName> },
    Fun { args: Vec<TypeName>, ret_ty: Box<TypeName> },
    Tuple { type_names: Vec<TypeName> }
}

impl TypeName {
    pub fn try_from_node_pos(node_pos: &ASTNodePos) -> Result<Self, TypeErr> {
        match &node_pos.node {
            // TODO implement generics
            ASTNode::Type { id, generics } => match &id.node {
                ASTNode::Id { lit } => Ok(TypeName::Single {
                    lit:      lit.clone(),
                    generics: generics
                        .iter()
                        .map(|generic| TypeName::try_from_node_pos(generic))
                        .collect::<Result<Vec<TypeName>, TypeErr>>()?
                }),
                _ => Err(TypeErr::new(Position::from(id.deref()), "Expected identifier"))
            },
            ASTNode::TypeTup { types } => Ok(TypeName::Tuple {
                type_names: types
                    .iter()
                    .map(|node_pos| TypeName::try_from_node_pos(node_pos))
                    .collect::<Result<Vec<TypeName>, TypeErr>>()?
            }),
            ASTNode::TypeFun { args, ret_ty } => Ok(TypeName::Fun {
                args:   args
                    .iter()
                    .map(|node_pos| TypeName::try_from_node_pos(node_pos))
                    .collect::<Result<Vec<TypeName>, TypeErr>>()?,
                ret_ty: Box::from(TypeName::try_from_node_pos(ret_ty)?)
            }),
            _ => Err(TypeErr::new(Position::from(node_pos), "Expected type variant"))
        }
    }
}
