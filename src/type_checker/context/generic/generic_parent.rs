use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::context::generic::function_arg::GenericFunctionArg;
use crate::type_checker::context::generic::parameter::GenericParameter;
use crate::type_checker::type_result::TypeErr;
use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub struct GenericParent {
    pub name:     String,
    pub pos:      Position,
    pub generics: Vec<GenericParameter>,
    pub args:     Vec<GenericFunctionArg>
}

impl TryFrom<&AST> for GenericParent {
    type Error = TypeErr;

    fn try_from(ast: &AST) -> Result<Self, Self::Error> {
        match &ast.node {
            Node::Parent { id, generics, args } => Ok(GenericParent {
                name:     match &id.node {
                    Node::Id { lit } => lit.clone(),
                    _ => return Err(TypeErr::new(&id.pos.clone(), "Expected identifier"))
                },
                pos:      ast.pos.clone(),
                generics: generics
                    .iter()
                    .map(GenericParameter::try_from)
                    .collect::<Result<_, _>>()?,
                args:     args
                    .iter()
                    .map(GenericFunctionArg::try_from)
                    .collect::<Result<_, _>>()?
            }),
            _ => Err(TypeErr::new(&ast.pos.clone(), "Expected parent"))
        }
    }
}