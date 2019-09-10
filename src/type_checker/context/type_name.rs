use std::convert::TryFrom;
use std::ops::Deref;

use crate::parser::ast::{ASTNode, ASTNodePos};
use crate::type_checker::type_result::TypeErr;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeName {
    Single { lit: String, generics: Vec<TypeName> },
    Fun { args: Vec<TypeName>, ret_ty: Box<TypeName> },
    Tuple { type_names: Vec<TypeName> }
}

impl From<&String> for TypeName {
    fn from(name: &String) -> Self { TypeName { lit: name.clone(), generics: vec![] } }
}

impl TryFrom<&ASTNodePos> for TypeName {
    type Error = TypeErr;

    fn try_from(node_pos: &ASTNodePos) -> Result<Self, Self::Error> {
        match &node_pos.node {
            ASTNode::Generic { id, .. } => TypeName::try_from(id.deref()),
            ASTNode::Type { id, generics } => match &id.node {
                ASTNode::Id { lit } => Ok(TypeName::Single {
                    lit:      lit.clone(),
                    generics: generics
                        .iter()
                        .map(TypeName::try_from)
                        .collect::<Result<Vec<TypeName>, TypeErr>>()?
                }),
                _ => Err(TypeErr::new(&id.position, "Expected identifier"))
            },
            ASTNode::TypeTup { types } => Ok(TypeName::Tuple {
                type_names: types
                    .iter()
                    .map(TypeName::try_from)
                    .collect::<Result<Vec<TypeName>, TypeErr>>()?
            }),
            ASTNode::TypeFun { args, ret_ty } => Ok(TypeName::Fun {
                args:   args
                    .iter()
                    .map(TypeName::try_from)
                    .collect::<Result<Vec<TypeName>, TypeErr>>()?,
                ret_ty: Box::from(TypeName::try_from(ret_ty.deref())?)
            }),
            _ => Err(TypeErr::new(&node_pos.position, "Expected type variant"))
        }
    }
}
