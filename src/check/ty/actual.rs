use std::convert::TryFrom;
use std::fmt;
use std::fmt::Display;
use std::ops::Deref;

use crate::check::context::name::Name;
use crate::check::result::{TypeErr, TypeResult};
use crate::check::ty::name::TypeName;
use crate::common::delimit::comma_delimited;
use crate::common::position::Position;
use crate::parse::ast::{Node, AST};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ActualType {
    Single { name: Name },
    Tuple { ty_names: Vec<TypeName> },
    AnonFun { args: Vec<TypeName>, ret_ty: Box<TypeName> }
}

impl Display for ActualType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ActualType::Single { name } => write!(f, "{}", name),
            ActualType::AnonFun { args, ret_ty } =>
                write!(f, "({}) -> {}", comma_delimited(args), ret_ty),
            ActualType::Tuple { ty_names } => write!(f, "({})", comma_delimited(ty_names))
        }
    }
}

impl TryFrom<&AST> for ActualType {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> TypeResult<ActualType> {
        match &ast.node {
            Node::Id { lit } => Ok(ActualType::Single { name: Name::from(lit) }),
            Node::Generic { id, .. } => ActualType::try_from(id.deref()),
            Node::Type { id, generics } => match &id.node {
                Node::Id { lit } => Ok(ActualType::Single {
                    name: Name::new(&lit.clone(), generics.iter().map(Name::try_from).collect()?)
                }),
                _ => Err(vec![TypeErr::new(&id.pos, "Expected identifier")])
            },
            Node::TypeTup { types } => Ok(ActualType::Tuple {
                ty_names: types.iter().map(TypeName::try_from).collect::<Result<_, _>>()?
            }),
            Node::TypeFun { args, ret_ty } => Ok(ActualType::AnonFun {
                args:   args.iter().map(TypeName::try_from).collect::<Result<_, _>>()?,
                ret_ty: Box::from(TypeName::try_from(ret_ty.deref())?)
            }),
            _ => Err(vec![TypeErr::new(&ast.pos, "Expected valid type variant")])
        }
    }
}
