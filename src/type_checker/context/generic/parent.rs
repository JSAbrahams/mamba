use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::context::generic::parameter::GenericParameter;
use crate::type_checker::context::generic::type_name::GenericActualTypeName;
use crate::type_checker::type_result::TypeErr;
use std::convert::TryFrom;

// TODO args should be literals or identifiers

#[derive(Debug, Clone)]
pub struct GenericParent {
    pub is_py_type: bool,
    pub name:       String,
    pub pos:        Position,
    pub generics:   Vec<GenericParameter>,
    pub args:       Vec<GenericActualTypeName>
}

impl TryFrom<&AST> for GenericParent {
    type Error = TypeErr;

    fn try_from(ast: &AST) -> Result<Self, Self::Error> {
        match &ast.node {
            // TODO infer types of arguments passed to parent
            // TODO use arguments
            Node::Parent { id, generics, .. } => Ok(GenericParent {
                is_py_type: false,
                name:       match &id.node {
                    Node::Id { lit } => lit.clone(),
                    _ => return Err(TypeErr::new(&id.pos.clone(), "Expected identifier"))
                },
                pos:        ast.pos.clone(),
                generics:   generics
                    .iter()
                    .map(GenericParameter::try_from)
                    .collect::<Result<_, _>>()?,
                args:       vec![]
            }),
            _ => Err(TypeErr::new(&ast.pos.clone(), "Expected parent"))
        }
    }
}
