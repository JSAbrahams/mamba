use std::convert::TryFrom;
use std::hash::Hash;

use crate::check::context::name::{DirectName, NameUnion};
use crate::check::result::{TypeErr, TypeResult};
use crate::common::position::Position;
use crate::parse::ast::{Node, AST};

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct GenericParent {
    pub is_py_type: bool,
    pub name:       DirectName,
    pub pos:        Position,
    pub args:       Vec<NameUnion>
}

impl TryFrom<&AST> for GenericParent {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> TypeResult<GenericParent> {
        match &ast.node {
            // TODO infer types of arguments passed to parent
            // TODO use arguments
            Node::Parent { ty, .. } => Ok(GenericParent {
                is_py_type: false,
                name:       DirectName::try_from(ty)?,
                pos:        ast.pos.clone(),
                args:       vec![]
            }),
            _ => {
                let msg = format!("Expected parent, was {}", ast.node);
                Err(vec![TypeErr::new(&ast.pos.clone(), &msg)])
            }
        }
    }
}
