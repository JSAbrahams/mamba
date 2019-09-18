use std::convert::TryFrom;
use std::fmt;
use std::fmt::Display;
use std::ops::Deref;

use crate::parser::ast::{Node, AST};
use crate::type_checker::type_result::TypeErr;

#[derive(Debug, Clone, PartialEq)]
pub enum GenericTypeName {
    Single { lit: String, generics: Vec<GenericTypeName> },
    Fun { args: Vec<GenericTypeName>, ret_ty: Box<GenericTypeName> },
    Tuple { type_names: Vec<GenericTypeName> }
}

impl Display for GenericTypeName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GenericTypeName::Single { lit, generics } if generics.is_empty() =>
                write!(f, "{}", lit),
            GenericTypeName::Single { lit, generics } =>
                write!(f, "{}<{}>", lit, comma_delimited(generics)?),
            GenericTypeName::Fun { args, ret_ty } =>
                write!(f, "({}) -> {}", comma_delimited(args)?, ret_ty),
            GenericTypeName::Tuple { type_names } => write!(f, "({})", comma_delimited(type_names)?)
        }
    }
}

fn comma_delimited(types: &[GenericTypeName]) -> Result<String, fmt::Error> {
    let mut res = String::new();
    for ty in types {
        res.push_str(format!("{}", ty).as_str());
        res.push(',');
        res.push(' ');
    }

    if res.len() > 1 {
        res.remove(res.len() - 1);
        res.remove(res.len() - 1);
    }
    Ok(res)
}

impl TryFrom<&AST> for GenericTypeName {
    type Error = TypeErr;

    fn try_from(node_pos: &AST) -> Result<Self, Self::Error> {
        match &node_pos.node {
            Node::Id { lit } =>
                Ok(GenericTypeName::Single { lit: lit.clone(), generics: vec![] }),
            Node::Generic { id, .. } => GenericTypeName::try_from(id.deref()),
            Node::Type { id, generics } => match &id.node {
                Node::Id { lit } => Ok(GenericTypeName::Single {
                    lit:      lit.clone(),
                    generics: generics
                        .iter()
                        .map(GenericTypeName::try_from)
                        .collect::<Result<_, _>>()?
                }),
                _ => Err(TypeErr::new(&id.pos, "Expected identifier"))
            },
            Node::TypeTup { types } => Ok(GenericTypeName::Tuple {
                type_names: types
                    .iter()
                    .map(GenericTypeName::try_from)
                    .collect::<Result<_, _>>()?
            }),
            Node::TypeFun { args, ret_ty } => Ok(GenericTypeName::Fun {
                args:   args.iter().map(GenericTypeName::try_from).collect::<Result<_, _>>()?,
                ret_ty: Box::from(GenericTypeName::try_from(ret_ty.deref())?)
            }),
            _ => Err(TypeErr::new(&node_pos.pos, "Expected type variant"))
        }
    }
}
