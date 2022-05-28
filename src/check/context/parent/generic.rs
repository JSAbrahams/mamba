use std::convert::TryFrom;
use std::hash::Hash;

use crate::check::name::truename::TrueName;
use crate::check::result::{TypeErr, TypeResult};
use crate::common::position::Position;
use crate::parse::ast::{AST, Node};

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct GenericParent {
    pub is_py_type: bool,
    pub name: TrueName,
    pub pos: Position,
}

impl TryFrom<&AST> for GenericParent {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> TypeResult<GenericParent> {
        match &ast.node {
            Node::Parent { ty, .. } => Ok(GenericParent {
                is_py_type: false,
                name: TrueName::try_from(ty)?,
                pos: ast.pos.clone(),
            }),
            _ => {
                let msg = format!("Expected parent, was {}", ast.node);
                Err(vec![TypeErr::new(&ast.pos.clone(), &msg)])
            }
        }
    }
}
