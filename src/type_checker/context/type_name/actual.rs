use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::Display;
use std::ops::Deref;

use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::environment::expression_type::actual_type::ActualType;
use crate::type_checker::type_result::{TypeErr, TypeResult};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ActualTypeName {
    Single { lit: String, generics: Vec<ActualTypeName> },
    Tuple { ty_names: Vec<TypeName> },
    AnonFun { args: Vec<TypeName>, ret_ty: Box<TypeName> }
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
                    generics: generics
                        .iter()
                        .map(ActualTypeName::try_from)
                        .collect::<Result<_, _>>()?
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

impl From<&str> for ActualTypeName {
    fn from(name: &str) -> Self {
        ActualTypeName::Single { lit: String::from(name), generics: vec![] }
    }
}

impl From<&ActualType> for ActualTypeName {
    fn from(actual_type: &ActualType) -> Self {
        match actual_type {
            ActualType::Single { ty } => ty.name.clone(),
            ActualType::Tuple { types } => ActualTypeName::Tuple {
                ty_names: types.iter().map(|ty| TypeName::from(ty)).collect()
            },
            ActualType::AnonFun { args, ret_ty } => ActualTypeName::AnonFun {
                args:   args.iter().map(|arg| TypeName::from(arg)).collect(),
                ret_ty: Box::new(TypeName::from(ret_ty.deref()))
            }
        }
    }
}

impl From<(&str, &Vec<ActualType>)> for ActualTypeName {
    fn from((name, generics): (&str, &Vec<ActualType>)) -> Self {
        ActualTypeName::Single {
            lit:      String::from(name.clone()),
            generics: generics.clone().iter().map(|g| g.name()).collect()
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

    pub fn as_single(&self, pos: &Position) -> TypeResult<(String, Vec<ActualTypeName>)> {
        match &self {
            ActualTypeName::Single { lit, generics } => Ok((lit.clone(), generics.clone())),
            _ => Err(vec![TypeErr::new(pos, &format!("Expected single but was {}", self))])
        }
    }

    pub fn substitute(
        &self,
        gens: &HashMap<String, ActualTypeName>,
        pos: &Position
    ) -> TypeResult<ActualTypeName> {
        match &self {
            ActualTypeName::Single { lit, generics } =>
                if let Some(subst) = gens.get(lit) {
                    match subst {
                        ActualTypeName::Single { generics: subs_generics, .. }
                            if generics.len() == subs_generics.len() =>
                            Ok(subst.clone()),
                        _ if generics.is_empty() => Ok(subst.clone()),
                        _ => {
                            let msg = format!("Cannot substitute {} with {}", &self, subst);
                            Err(vec![TypeErr::new(pos, &msg)])
                        }
                    }
                } else {
                    Ok(ActualTypeName::Single {
                        lit:      lit.clone(),
                        generics: generics
                            .iter()
                            .map(|ty| ty.substitute(gens, pos))
                            .collect::<Result<_, _>>()?
                    })
                },
            ActualTypeName::Tuple { ty_names } => Ok(ActualTypeName::Tuple {
                ty_names: ty_names
                    .iter()
                    .map(|ty| ty.substitute(gens, pos))
                    .collect::<Result<_, _>>()?
            }),
            ActualTypeName::AnonFun { args, ret_ty } => Ok(ActualTypeName::AnonFun {
                args:   args.iter().map(|ty| ty.substitute(gens, pos)).collect::<Result<_, _>>()?,
                ret_ty: Box::from(ret_ty.substitute(gens, pos)?)
            })
        }
    }
}
