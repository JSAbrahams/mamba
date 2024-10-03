use std::convert::TryFrom;
use std::ops::Deref;

use crate::check::name::string_name::StringName;
use crate::check::name::true_name::TrueName;
use crate::check::name::{Name, Nullable, TupleCallable};
use crate::check::result::{TypeErr, TypeResult};
use crate::parse::ast::{Node, AST};

impl TryFrom<&Box<AST>> for TrueName {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &Box<AST>) -> Result<Self, Self::Error> {
        TrueName::try_from(ast.deref())
    }
}

impl TryFrom<&AST> for TrueName {
    type Error = Vec<TypeErr>;

    /// Try to construct Name from AST.
    ///
    /// In the case of Generics, isa field is ignored and we only look at the
    /// true_name of the generic itself.
    fn try_from(ast: &AST) -> TypeResult<TrueName> {
        match &ast.node {
            Node::Id { lit } => Ok(TrueName::from(&StringName::from(lit.as_str()))),
            Node::Tuple { elements } => {
                let elements: Vec<Name> = elements
                    .iter()
                    .map(Name::try_from)
                    .collect::<Result<_, _>>()?;
                Ok(TrueName::tuple(elements.as_slice()))
            }
            Node::QuestionOp { expr } => Ok(TrueName::try_from(expr)?.as_nullable()),
            Node::Type { .. } => Ok(TrueName::from(&StringName::try_from(ast)?)),
            Node::TypeTup { types } => {
                let names: Vec<Name> =
                    types.iter().map(Name::try_from).collect::<Result<_, _>>()?;
                Ok(TrueName::tuple(names.as_slice()))
            }
            Node::TypeFun { args, ret_ty } => Ok(TrueName::callable(
                args.iter()
                    .map(Name::try_from)
                    .collect::<Result<Vec<Name>, _>>()?
                    .as_slice(),
                &Name::try_from(ret_ty.deref())?,
            )),
            Node::TypeUnion { .. } => Err(vec![TypeErr::new(
                ast.pos,
                "Expected single type name but was union",
            )]),
            Node::Generic { id, .. } => TrueName::try_from(id),
            Node::FunctionCall { name, .. } => TrueName::try_from(name),
            Node::Parent { ty, .. } => TrueName::try_from(ty),
            _ => {
                let msg = format!("Expected type name, was {}", ast.node);
                Err(vec![TypeErr::new(ast.pos, &msg)])
            }
        }
    }
}
