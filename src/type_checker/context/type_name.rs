use std::convert::TryFrom;
use std::ops::Deref;

use crate::parser::ast::{Node, AST};
use crate::type_checker::type_result::TypeErr;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeName {
    Single { lit: String, generics: Vec<TypeName> },
    Fun { args: Vec<TypeName>, ret_ty: Box<TypeName> },
    Tuple { type_names: Vec<TypeName> }
}

impl From<&String> for TypeName {
    fn from(name: &String) -> Self { TypeName::Single { lit: name.clone(), generics: vec![] } }
}

impl TryFrom<&AST> for TypeName {
    type Error = TypeErr;

    fn try_from(node_pos: &AST) -> Result<Self, Self::Error> {
        match &node_pos.node {
            Node::Generic { id, .. } => TypeName::try_from(id.deref()),
            Node::Type { id, generics } => match &id.node {
                Node::Id { lit } => Ok(TypeName::Single {
                    lit:      lit.clone(),
                    generics: generics
                        .iter()
                        .map(TypeName::try_from)
                        .collect::<Result<Vec<TypeName>, TypeErr>>()?
                }),
                _ => Err(TypeErr::new(&id.pos, "Expected identifier"))
            },
            Node::TypeTup { types } => Ok(TypeName::Tuple {
                type_names: types
                    .iter()
                    .map(TypeName::try_from)
                    .collect::<Result<Vec<TypeName>, TypeErr>>()?
            }),
            Node::TypeFun { args, ret_ty } => Ok(TypeName::Fun {
                args:   args
                    .iter()
                    .map(TypeName::try_from)
                    .collect::<Result<Vec<TypeName>, TypeErr>>()?,
                ret_ty: Box::from(TypeName::try_from(ret_ty.deref())?)
            }),
            _ => Err(TypeErr::new(&node_pos.pos, "Expected type variant"))
        }
    }
}
