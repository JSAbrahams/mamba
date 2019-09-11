use std::convert::TryFrom;
use std::ops::Deref;

use crate::parser::ast::{Node, AST};
use crate::type_checker::context::generic_type_name::GenericTypeName;
use crate::type_checker::type_result::TypeErr;

#[derive(Debug, Clone)]
pub struct GenericParameter {
    pub name:   String,
    pub parent: Option<GenericTypeName>
}

impl TryFrom<&AST> for GenericParameter {
    type Error = TypeErr;

    fn try_from(ast: &AST) -> Result<Self, Self::Error> {
        match &ast.node {
            Node::Generic { id, isa } => match isa {
                Some(isa) => Ok(GenericParameter {
                    name:   parameter_name(id.deref())?,
                    parent: Some(GenericTypeName::try_from(isa.deref())?)
                }),
                None => Ok(GenericParameter { name: parameter_name(id.deref())?, parent: None })
            },
            _ => Err(TypeErr::new(&ast.pos.clone(), "Expected generic"))
        }
    }
}

fn parameter_name(ast: &AST) -> Result<String, TypeErr> {
    match &ast.node {
        Node::Id { lit } => Ok(lit.clone()),
        _ => Err(TypeErr::new(&ast.pos, "Expected parameter name"))
    }
}
