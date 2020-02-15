use std::collections::HashSet;
use std::convert::TryFrom;
use std::iter::FromIterator;

use crate::check::context::name::{Name, NameUnion};
use crate::check::result::{TypeErr, TypeResult};
use crate::parse::ast::{Node, AST};

impl TryFrom<&AST> for NameUnion {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> Result<Self, Self::Error> {
        let names = if let Node::TypeUnion { types } = &ast.node {
            types.iter().map(Name::try_from).collect::<Result<_, _>>()?
        } else {
            HashSet::from_iter(vec![Name::try_from(ast)?].iter())
        };
        Ok(NameUnion { names })
    }
}

impl TryFrom<&AST> for Name {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> TypeResult<Name> {
        match &ast.node {
            Node::Id { lit } => Ok(Name::Single(lit.clone(), vec![])),
            Node::Tuple { elements } => {
                let names = elements.iter().map(|e| Name::try_from(e)).collect()?;
                Ok(Name::Tuple(names))
            }
            Node::Type { id, generics } => match &id.node {
                Node::Id { lit } => {
                    let generics = generics.iter().map(Name::try_from).collect::<Result<_, _>>()?;
                    Ok(Name::Single(lit.clone(), generics))
                }
                _ => Err(vec![TypeErr::new(&id.pos, "Expected identifier")])
            },
            _ => Err(vec![TypeErr::new(&ast.pos, "Expected name")])
        }
    }
}
