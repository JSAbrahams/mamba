use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::Display;
use std::ops::Deref;

use crate::check::checker_result::{TypeErr, TypeResult};
use crate::check::ty::concrete::actual::ActualType;
use crate::check::ty::name::TypeName;
use crate::common::delimit::comma_delimited;
use crate::common::position::Position;
use crate::parse::ast::{Node, AST};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ActualTypeName {
    Single { lit: String, generics: Vec<TypeName> },
    Tuple { ty_names: Vec<TypeName> },
    AnonFun { args: Vec<TypeName>, ret_ty: Box<TypeName> }
}

impl Display for ActualTypeName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ActualTypeName::Single { lit, generics } if generics.is_empty() => write!(f, "{}", lit),
            ActualTypeName::Single { lit, generics } =>
                write!(f, "{}[{}]", lit, comma_delimited(generics)),
            ActualTypeName::AnonFun { args, ret_ty } =>
                write!(f, "({}) -> {}", comma_delimited(args), ret_ty),
            ActualTypeName::Tuple { ty_names } => write!(f, "({})", comma_delimited(ty_names))
        }
    }
}

impl TryFrom<&AST> for ActualTypeName {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> TypeResult<ActualTypeName> {
        match &ast.node {
            Node::Id { lit } =>
                Ok(ActualTypeName::Single { lit: lit.clone(), generics: vec![] }),
            Node::Generic { id, .. } => ActualTypeName::try_from(id.deref()),
            Node::Type { id, generics } => match &id.node {
                Node::Id { lit } => Ok(ActualTypeName::Single {
                    lit:      lit.clone(),
                    generics: generics.iter().map(TypeName::try_from).collect::<Result<_, _>>()?
                }),
                _ => Err(vec![TypeErr::new(&id.pos, "Expected identifier")])
            },
            Node::TypeTup { types } => Ok(ActualTypeName::Tuple {
                ty_names: types.iter().map(TypeName::try_from).collect::<Result<_, _>>()?
            }),
            Node::TypeFun { args, ret_ty } => Ok(ActualTypeName::AnonFun {
                args:   args.iter().map(TypeName::try_from).collect::<Result<_, _>>()?,
                ret_ty: Box::from(TypeName::try_from(ret_ty.deref())?)
            }),
            _ => Err(vec![TypeErr::new(&ast.pos, "Expected type variant")])
        }
    }
}

impl From<&ActualType> for ActualTypeName {
    fn from(actual_type: &ActualType) -> Self {
        match actual_type {
            ActualType::Single { ty } => ty.name.clone(),
            ActualType::Tuple { types } =>
                ActualTypeName::Tuple { ty_names: types.iter().map(TypeName::from).collect() },
            ActualType::AnonFun { args, ret_ty } => ActualTypeName::AnonFun {
                args:   args.iter().map(TypeName::from).collect(),
                ret_ty: Box::new(TypeName::from(ret_ty.deref()))
            }
        }
    }
}

impl ActualTypeName {
    pub fn new(lit: &str, generics: &[TypeName]) -> ActualTypeName {
        ActualTypeName::Single { lit: String::from(lit), generics: generics.to_vec() }
    }

    pub fn name(&self, pos: &Position) -> TypeResult<String> {
        match self {
            ActualTypeName::Single { lit, .. } => Ok(lit.clone()),
            _ => Err(vec![TypeErr::new(pos, &format!("{} does not have name", self))])
        }
    }

    pub fn substitute(
        &self,
        gens: &HashMap<String, TypeName>,
        pos: &Position
    ) -> TypeResult<ActualTypeName> {
        Ok(match self {
            ActualTypeName::Single { lit, generics } =>
                if generics.is_empty() {
                    // if no generic do direct substitution
                    // TODO what about direct substitution of unions?
                    match gens.get(lit).cloned() {
                        Some(ty_name) => ty_name.single(pos)?,
                        None => self.clone()
                    }
                } else {
                    //
                    ActualTypeName::Single {
                        lit:      match gens.get(lit).cloned() {
                            Some(ty_name) => ty_name.single(pos)?.name(pos)?,
                            None => lit.clone()
                        },
                        generics: generics
                            .iter()
                            .map(|g| g.substitute(gens, pos))
                            .collect::<Result<_, _>>()?
                    }
                },
            ActualTypeName::Tuple { ty_names } => ActualTypeName::Tuple {
                ty_names: ty_names
                    .iter()
                    .map(|t| t.substitute(gens, pos))
                    .collect::<Result<_, _>>()?
            },
            ActualTypeName::AnonFun { args, ret_ty } => ActualTypeName::AnonFun {
                args:   args.iter().map(|a| a.substitute(gens, pos)).collect::<Result<_, _>>()?,
                ret_ty: Box::from(ret_ty.substitute(gens, pos)?)
            }
        })
    }
}
