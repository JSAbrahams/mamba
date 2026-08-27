use std::convert::TryFrom;
use std::hash::Hash;

use crate::check::name::true_name::TrueName;
use crate::check::result::{TypeErr, TypeResult};
use crate::common::position::Position;
use crate::parse::ast::{Node, AST};

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
            Node::Parent { ty, .. } => match &ty.node {
                Node::TypeFun { .. } => {
                    let msg = "A class or trait cannot inherit from a function type";
                    Err(vec![TypeErr::new(ty.pos, msg)])
                }
                Node::TypeTup { .. } => {
                    let msg = "A class or trait cannot inherit from a tuple type";
                    Err(vec![TypeErr::new(ty.pos, msg)])
                }
                _ => Ok(GenericParent {
                    is_py_type: false,
                    name: TrueName::try_from(ty)?,
                    pos: ast.pos,
                }),
            },
            _ => {
                let msg = format!("Expected parent, was {}", ast.node);
                Err(vec![TypeErr::new(ast.pos, &msg)])
            }
        }
    }
}
