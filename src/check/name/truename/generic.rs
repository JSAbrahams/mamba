use std::convert::TryFrom;
use std::ops::Deref;

use crate::check::name::namevariant::NameVariant;
use crate::check::name::stringname::StringName;
use crate::check::name::truename::TrueName;
use crate::check::name::AsNullable;
use crate::check::name::Name;
use crate::check::result::{TypeErr, TypeResult};
use crate::parse::ast::{Node, AST};

impl TryFrom<&Box<AST>> for TrueName {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &Box<AST>) -> Result<Self, Self::Error> { TrueName::try_from(ast.deref()) }
}

impl TryFrom<&AST> for TrueName {
    type Error = Vec<TypeErr>;

    /// Try to construct Name from AST.
    ///
    /// In the case of Generics, isa field is ignored and we only look at the
    /// truename of the generic itself.
    fn try_from(ast: &AST) -> TypeResult<TrueName> {
        match &ast.node {
            Node::Id { lit } => Ok(TrueName::from(&StringName::from(lit.as_str()))),
            Node::Tuple { elements } => {
                let elements = elements.iter().map(Name::try_from).collect::<Result<_, _>>()?;
                Ok(TrueName::from(&NameVariant::Tuple(elements)))
            }
            Node::QuestionOp { expr } => Ok(TrueName::try_from(expr)?.as_nullable()),
            Node::Type { id, .. } => Ok(TrueName::try_from(id)?),
            Node::TypeTup { types } => {
                let names = types.iter().map(Name::try_from).collect::<Result<_, _>>()?;
                Ok(TrueName::from(&NameVariant::Tuple(names)))
            }
            Node::TypeFun { args, ret_ty } => Ok(TrueName::from(&NameVariant::Fun(
                args.iter().map(Name::try_from).collect::<Result<_, _>>()?,
                Box::from(Name::try_from(ret_ty.deref())?)
            ))),
            Node::TypeUnion { .. } =>
                Err(vec![TypeErr::new(&ast.pos, "Expected single type name but was union")]),
            Node::Generic { id, .. } => TrueName::try_from(id),
            Node::FunctionCall { name, .. } => TrueName::try_from(name),
            Node::Parent { ty, .. } => TrueName::try_from(ty),
            _ => {
                let msg = format!("Expected type name, was {}", ast.node);
                Err(vec![TypeErr::new(&ast.pos, &msg)])
            }
        }
    }
}
