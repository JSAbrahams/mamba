use crate::parser::ast::{Node, AST};
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use std::convert::TryFrom;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct GenericParameter {
    pub is_py_type: bool,
    pub name:       String,
    pub parent:     Option<TypeName>
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
