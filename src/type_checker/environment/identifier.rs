use std::collections::HashSet;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::iter::FromIterator;

use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::environment::expression_type::actual_type::ActualType;
use crate::type_checker::environment::expression_type::ExpressionType;
use crate::type_checker::type_result::{TypeErr, TypeResult};

#[derive(Clone, Debug)]
pub struct Identifier {
    lit:   Option<(bool, String)>,
    names: Vec<Identifier>
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some((mutable, lit)) = &self.lit {
            write!(f, "{}{}", if *mutable { "mut " } else { "" }, lit.clone())
        } else {
            write!(
                f,
                "({})",
                {
                    let mut string = String::new();
                    self.names.iter().for_each(|name| string.push_str(&format!("{}, ", name)));
                    string.remove(string.len() - 2);
                    string
                }
                .trim_end()
            )
        }
    }
}

impl TryFrom<&AST> for Identifier {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> TypeResult<Identifier> {
        match &ast.node {
            // TODO add mutable field to identifier
            Node::Id { lit } => Ok(Identifier::from(lit.as_str())),
            Node::Tuple { elements } =>
                Ok(Identifier::from(&elements.iter().map(Identifier::try_from).collect::<Result<
                    Vec<Identifier>,
                    _
                >>(
                )?)),
            _ => Err(vec![TypeErr::new(&ast.pos, "Expected id or tuple of id's")])
        }
    }
}

impl From<&str> for Identifier {
    fn from(name: &str) -> Self {
        // TODO use mutable field from identifier
        Identifier { lit: Some((false, String::from(name))), names: vec![] }
    }
}

impl From<&Vec<Identifier>> for Identifier {
    fn from(names: &Vec<Identifier>) -> Self { Identifier { lit: None, names: names.to_vec() } }
}

pub fn match_name(
    identifier: &Identifier,
    expr_ty: &ExpressionType,
    pos: &Position
) -> TypeResult<HashSet<(bool, String, ExpressionType)>> {
    match expr_ty {
        ExpressionType::Single { mut_ty } => match &mut_ty.actual_ty {
            ActualType::Single { .. } | ActualType::AnonFun { .. } =>
                if let Some((mutable, id)) = &identifier.lit {
                    Ok(HashSet::from_iter(vec![(*mutable, id.clone(), expr_ty.clone())].to_vec()))
                } else {
                    let msg = format!("Cannot match {} with type {}", identifier, expr_ty);
                    Err(vec![TypeErr::new(pos, &msg)])
                },
            ActualType::Tuple { types } =>
                if let Some((mutable, id)) = &identifier.lit {
                    Ok(HashSet::from_iter(vec![(*mutable, id.clone(), expr_ty.clone())].to_vec()))
                } else if types.len() == identifier.names.len() {
                    let sets: Vec<HashSet<_>> = identifier
                        .names
                        .iter()
                        .zip(types)
                        .map(|(identifier, expr_ty)| match_name(&identifier, expr_ty, pos))
                        .collect::<Result<_, _>>()?;
                    Ok(sets.into_iter().flatten().collect())
                } else {
                    let msg = format!(
                        "Cannot iterate over ({:?}) with tuple of size {}",
                        types,
                        identifier.names.len()
                    );
                    Err(vec![TypeErr::new(pos, &msg)])
                },
        },
        ExpressionType::Union { union } => unimplemented!()
    }
}
