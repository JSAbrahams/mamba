use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::Display;

use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::environment::expression_type::actual_type::ActualType;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use std::ops::Deref;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ActualTypeName {
    Single { lit: String, generics: Vec<ActualTypeName> },
    Tuple { ty_names: Vec<ActualTypeName> },
    AnonFun { args: Vec<ActualTypeName>, ret_ty: Box<ActualTypeName> }
}

impl Display for ActualTypeName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ActualTypeName::Single { lit, generics } if generics.is_empty() => write!(f, "{}", lit),
            ActualTypeName::Single { lit, generics } => write!(f, "{}<{:#?}>", lit, generics),
            ActualTypeName::AnonFun { args, ret_ty } => write!(f, "({:#?}) -> {}", args, ret_ty),
            ActualTypeName::Tuple { ty_names } => write!(f, "({:#?})", ty_names)
        }
    }
}

impl From<&str> for ActualTypeName {
    fn from(name: &str) -> Self {
        ActualTypeName::Single { lit: String::from(name), generics: vec![] }
    }
}

impl TryFrom<&AST> for ActualTypeName {
    type Error = TypeErr;

    fn try_from(ast: &AST) -> Result<Self, Self::Error> {
        match &ast.node {
            Node::Id { lit } =>
                Ok(ActualTypeName::Single { lit: lit.clone(), generics: vec![] }),
            Node::Generic { id, .. } => ActualTypeName::try_from(id.deref()),
            Node::Type { id, generics } => match &id.node {
                Node::Id { lit } => Ok(ActualTypeName::Single {
                    lit:      lit.clone(),
                    generics: generics
                        .iter()
                        .map(ActualTypeName::try_from)
                        .collect::<Result<_, _>>()?
                }),
                _ => Err(TypeErr::new(&id.pos, "Expected identifier"))
            },
            Node::TypeTup { types } => Ok(ActualTypeName::Tuple {
                ty_names: types.iter().map(ActualTypeName::try_from).collect::<Result<_, _>>()?
            }),
            Node::TypeFun { args, ret_ty } => Ok(ActualTypeName::AnonFun {
                args:   args.iter().map(ActualTypeName::try_from).collect::<Result<_, _>>()?,
                ret_ty: Box::from(ActualTypeName::try_from(ret_ty.deref())?)
            }),
            _ => Err(TypeErr::new(&ast.pos, "Expected type variant"))
        }
    }
}

impl From<&ActualType> for ActualTypeName {
    fn from(actual_type: &ActualType) -> Self {
        match actual_type {
            ActualType::Single { ty } => ty.name.clone(),
            ActualType::Tuple { types } => ActualTypeName::Tuple {
                ty_names: types.iter().map(|ty| ActualTypeName::from(&ty.actual_ty)).collect()
            },
            ActualType::AnonFun { args, ret_ty } => ActualTypeName::AnonFun {
                args:   args.iter().map(|arg| ActualTypeName::from(&arg.actual_ty)).collect(),
                ret_ty: Box::new(ActualTypeName::from(&ret_ty.deref().actual_ty))
            }
        }
    }
}

impl ActualTypeName {
    pub fn new(lit: &str, generics: &[ActualTypeName]) -> ActualTypeName {
        ActualTypeName::Single { lit: String::from(lit), generics: Vec::from(generics) }
    }

    pub fn name(&self, pos: &Position) -> TypeResult<String> {
        match self {
            ActualTypeName::Single { lit, .. } => Ok(lit.clone()),
            _ => Err(vec![TypeErr::new(pos, "Type does not have name")])
        }
    }

    pub fn substitute(
        &self,
        generics: &HashMap<String, ActualTypeName>,
        pos: &Position
    ) -> TypeResult<ActualTypeName> {
        unimplemented!()
    }
}
