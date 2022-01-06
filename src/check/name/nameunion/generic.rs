use std::collections::HashSet;
use std::convert::TryFrom;
use std::ops::Deref;

use crate::check::name::nameunion::NameUnion;
use crate::check::name::truename::TrueName;
use crate::check::result::TypeErr;
use crate::parse::ast::{AST, Node};

impl TryFrom<&Box<AST>> for NameUnion {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &Box<AST>) -> Result<Self, Self::Error> { NameUnion::try_from(ast.deref()) }
}

impl TryFrom<&AST> for NameUnion {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> Result<Self, Self::Error> {
        let names = if let Node::TypeUnion { types } = &ast.node {
            types.iter().map(TrueName::try_from).collect::<Result<_, _>>()?
        } else {
            vec![TrueName::try_from(ast)?].into_iter().collect::<HashSet<_>>()
        };
        Ok(NameUnion { names })
    }
}

impl TryFrom<&Vec<AST>> for NameUnion {
    type Error = Vec<TypeErr>;

    fn try_from(asts: &Vec<AST>) -> Result<Self, Self::Error> {
        let names: Vec<TrueName> = asts.iter().map(TrueName::try_from).collect::<Result<_, _>>()?;
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
