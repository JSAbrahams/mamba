use std::convert::TryFrom;
use std::fmt::{Display, Error, Formatter};
use std::ops::Deref;

use crate::check::checker_result::{TypeErr, TypeResult};
use crate::check::ty::name::TypeName;
use crate::parse::ast::{Node, AST};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct GenericParameter {
    pub is_py_type: bool,
    pub name:       String,
    pub parent:     Option<TypeName>
}

impl Display for GenericParameter {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "{}{}",
            self.name,
            if let Some(parent) = &self.parent {
                format!(" isa {}", parent)
            } else {
                String::new()
            }
        )
    }
}

impl TryFrom<&AST> for GenericParameter {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> TypeResult<GenericParameter> {
        match &ast.node {
            Node::Generic { id, isa } => match isa {
                Some(isa) => Ok(GenericParameter {
                    is_py_type: false,
                    name:       parameter_name(id.deref())?,
                    parent:     Some(TypeName::try_from(isa.deref())?)
                }),
                None => Ok(GenericParameter {
                    is_py_type: false,
                    name:       parameter_name(id.deref())?,
                    parent:     None
                })
            },
            _ => Err(vec![TypeErr::new(&ast.pos.clone(), "Expected generic")])
        }
    }
}

fn parameter_name(ast: &AST) -> Result<String, TypeErr> {
    match &ast.node {
        Node::Id { lit } => Ok(lit.clone()),
        _ => Err(TypeErr::new(&ast.pos, "Expected parameter name"))
    }
}
