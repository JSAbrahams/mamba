use std::convert::TryFrom;
use std::fmt;
use std::fmt::Display;
use std::ops::Deref;

use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::type_result::{TypeErr, TypeResult};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GenericActualTypeName {
    Single { lit: String, generics: Vec<GenericActualTypeName> },
    Fun { args: Vec<GenericActualTypeName>, ret_ty: Box<GenericActualTypeName> },
    Tuple { ty_names: Vec<GenericActualTypeName> }
}

impl GenericActualTypeName {
    pub fn new(name: &str) -> GenericActualTypeName {
        GenericActualTypeName::Single { lit: String::from(name), generics: vec![] }
    }

    pub fn name(&self, pos: &Position) -> TypeResult<String> {
        match self {
            GenericActualTypeName::Single { lit, .. } => Ok(lit.clone()),
            _ => Err(vec![TypeErr::new(pos, "Type does not have name")])
        }
    }
}

impl Display for GenericActualTypeName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GenericActualTypeName::Single { lit, generics } if generics.is_empty() =>
                write!(f, "{}", lit),
            GenericActualTypeName::Single { lit, generics } =>
                write!(f, "{}<{:#?}>", lit, generics),
            GenericActualTypeName::Fun { args, ret_ty } => write!(f, "({:#?}) -> {}", args, ret_ty),
            GenericActualTypeName::Tuple { ty_names } => write!(f, "({:#?})", ty_names)
        }
    }
}

impl TryFrom<&AST> for GenericActualTypeName {
    type Error = TypeErr;

    fn try_from(ast: &AST) -> Result<Self, Self::Error> {
        match &ast.node {
            Node::Id { lit } =>
                Ok(GenericActualTypeName::Single { lit: lit.clone(), generics: vec![] }),
            Node::Generic { id, .. } => GenericActualTypeName::try_from(id.deref()),
            Node::Type { id, generics } => match &id.node {
                Node::Id { lit } => Ok(GenericActualTypeName::Single {
                    lit:      lit.clone(),
                    generics: generics
                        .iter()
                        .map(GenericActualTypeName::try_from)
                        .collect::<Result<_, _>>()?
                }),
                _ => Err(TypeErr::new(&id.pos, "Expected identifier"))
            },
            Node::TypeTup { types } => Ok(GenericActualTypeName::Tuple {
                ty_names: types
                    .iter()
                    .map(GenericActualTypeName::try_from)
                    .collect::<Result<_, _>>()?
            }),
            Node::TypeFun { args, ret_ty } => Ok(GenericActualTypeName::Fun {
                args:   args
                    .iter()
                    .map(GenericActualTypeName::try_from)
                    .collect::<Result<_, _>>()?,
                ret_ty: Box::from(GenericActualTypeName::try_from(ret_ty.deref())?)
            }),
            _ => Err(TypeErr::new(&ast.pos, "Expected type variant"))
        }
    }
}
