use std::convert::TryFrom;
use std::ops::Deref;

use crate::check::name::stringname::StringName;
use crate::check::name::Name;
use crate::check::result::TypeErr;
use crate::parse::ast::{Node, AST};

impl TryFrom<&Box<AST>> for StringName {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &Box<AST>) -> Result<Self, Self::Error> { StringName::try_from(ast.deref()) }
}

impl TryFrom<&AST> for StringName {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> Result<Self, Self::Error> {
        match &ast.node {
            Node::Id { lit } => Ok(StringName::from(lit.as_str())),
            Node::Type { id, generics } => match &id.node {
                Node::Id { lit } => {
                    let generics: Vec<Name> =
                        generics.iter().map(Name::try_from).collect::<Result<_, _>>()?;
                    Ok(StringName::new(lit, &generics))
                }
                _ => Err(vec![TypeErr::new(
                    &id.pos,
                    &format!("Expected identifier, was {}", ast.node)
                )])
            },
            _ => {
                let msg = format!("Expected class name, was {}", ast.node);
                Err(vec![TypeErr::new(&ast.pos, &msg)])
            }
        }
    }
}
