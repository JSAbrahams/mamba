use std::collections::HashSet;
use std::convert::TryFrom;
use std::iter::FromIterator;
use std::ops::Deref;

use crate::check::context::name::{AsNullable, DirectName, Name, NameUnion, NameVariant};
use crate::check::result::{TypeErr, TypeResult};
use crate::parse::ast::{Node, AST};

impl TryFrom<&Box<AST>> for DirectName {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &Box<AST>) -> Result<Self, Self::Error> { DirectName::try_from(ast.deref()) }
}

impl TryFrom<&AST> for DirectName {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> Result<Self, Self::Error> {
        match &ast.node {
            Node::Id { lit } => Ok(DirectName::from(lit.as_str())),
            Node::Parent { ty, .. } => DirectName::try_from(ty),
            Node::Type { id, generics } => match &id.node {
                Node::Id { lit } => {
                    let generics: Vec<NameUnion> =
                        generics.iter().map(NameUnion::try_from).collect::<Result<_, _>>()?;
                    Ok(DirectName::new(lit, &generics))
                }
                _ => Err(vec![TypeErr::new(&id.pos, "Expected identifier")])
            },
            _ => {
                let msg = format!("Expected class name, was {}", ast.node);
                Err(vec![TypeErr::new(&ast.pos, &msg)])
            }
        }
    }
}

impl TryFrom<&Box<AST>> for Name {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &Box<AST>) -> Result<Self, Self::Error> { Name::try_from(ast.deref()) }
}

impl TryFrom<&AST> for Name {
    type Error = Vec<TypeErr>;

    /// Try to construct Name from AST.
    ///
    /// In the case of Generics, isa field is ignored and we only look at the
    /// name of the generic itself.
    fn try_from(ast: &AST) -> TypeResult<Name> {
        match &ast.node {
            Node::Id { lit } => Ok(Name::from(&DirectName::from(lit.as_str()))),
            Node::QuestionOp { expr } => Ok(Name::try_from(expr)?.as_nullable()),
            Node::Type { .. } => Ok(Name::from(&DirectName::try_from(ast)?)),
            Node::TypeTup { types } => {
                let names = types.iter().map(NameUnion::try_from).collect::<Result<_, _>>()?;
                Ok(Name::from(&NameVariant::Tuple(names)))
            }
            Node::TypeFun { args, ret_ty } => Ok(Name::from(&NameVariant::Fun(
                args.iter().map(NameUnion::try_from).collect::<Result<_, _>>()?,
                Box::from(NameUnion::try_from(ret_ty.deref())?)
            ))),
            Node::TypeUnion { .. } =>
                Err(vec![TypeErr::new(&ast.pos, "Expected single name but was union")]),
            Node::Generic { id, .. } => Name::try_from(id),
            _ => {
                let msg = format!("Expected name, was {}", ast.node);
                Err(vec![TypeErr::new(&ast.pos, &msg)])
            }
        }
    }
}

impl TryFrom<&Box<AST>> for NameUnion {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &Box<AST>) -> Result<Self, Self::Error> { NameUnion::try_from(ast.deref()) }
}

impl TryFrom<&AST> for NameUnion {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> Result<Self, Self::Error> {
        let names = if let Node::TypeUnion { types } = &ast.node {
            types.iter().map(Name::try_from).collect::<Result<_, _>>()?
        } else {
            HashSet::from_iter(vec![Name::try_from(ast)?].into_iter())
        };
        Ok(NameUnion { names })
    }
}

impl TryFrom<&Vec<AST>> for NameUnion {
    type Error = Vec<TypeErr>;

    fn try_from(asts: &Vec<AST>) -> Result<Self, Self::Error> {
        let names: Vec<Name> = asts.iter().map(Name::try_from).collect::<Result<_, _>>()?;
        if let Some(first) = names.first() {
            let mut name_union = NameUnion::from(first);
            names.iter().for_each(|name| {
                name_union.names.insert(name.clone());
            });
            Ok(name_union)
        } else {
            Ok(NameUnion::empty())
        }
    }
}
