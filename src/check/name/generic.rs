use std::collections::HashSet;
use std::convert::TryFrom;
use std::ops::Deref;

use crate::check::name::{Empty, Name};
use crate::check::name::true_name::TrueName;
use crate::check::result::TypeErr;
use crate::parse::ast::{AST, Node};

impl TryFrom<&Box<AST>> for Name {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &Box<AST>) -> Result<Self, Self::Error> { Name::try_from(ast.deref()) }
}

impl TryFrom<&AST> for Name {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> Result<Self, Self::Error> {
        let names = if let Node::TypeUnion { types } = &ast.node {
            types.iter().map(TrueName::try_from).collect::<Result<_, _>>()?
        } else {
            vec![TrueName::try_from(ast)?].into_iter().collect::<HashSet<_>>()
        };
        Ok(Name { names, is_interchangeable: false })
    }
}

impl TryFrom<&Vec<AST>> for Name {
    type Error = Vec<TypeErr>;

    fn try_from(asts: &Vec<AST>) -> Result<Self, Self::Error> {
        let names: Vec<TrueName> = asts.iter().map(TrueName::try_from).collect::<Result<_, _>>()?;
        if let Some(first) = names.first() {
            let mut name_union = Name::from(first);
            names.iter().for_each(|name| {
                name_union.names.insert(name.clone());
            });
            Ok(name_union)
        } else {
            Ok(Name::empty())
        }
    }
}
