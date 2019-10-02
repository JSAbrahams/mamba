use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::context::type_name::generic::GenericTypeName;
use crate::type_checker::type_result::TypeErr;
use std::convert::TryFrom;
use std::ops::Deref;

#[derive(Debug, Clone, Eq)]
pub struct GenericField {
    pub is_py_type: bool,
    pub name:       String,
    pub pos:        Position,
    pub private:    bool,
    pub mutable:    bool,
    pub ty:         Option<GenericTypeName>
}

impl TryFrom<&AST> for GenericField {
    type Error = TypeErr;

    fn try_from(ast: &AST) -> Result<Self, Self::Error> {
        match &ast.node {
            // TODO do something with forward
            Node::VariableDef { private, id_maybe_type, .. } => {
                let (name, mutable, ty) = match &id_maybe_type.node {
                    Node::IdType { id, mutable, _type } =>
                        (field_name(id.deref())?, *mutable, match _type {
                            Some(_ty) => Some(GenericTypeName::try_from(_ty.deref())?),
                            None => None
                        }),
                    _ => return Err(TypeErr::new(&id_maybe_type.pos, "Expected identifier"))
                };

                Ok(GenericField {
                    is_py_type: false,
                    name,
                    mutable,
                    pos: ast.pos.clone(),
                    private: *private,
                    ty
                })
            }
            _ => Err(TypeErr::new(&ast.pos, "Expected variable"))
        }
    }
}

fn field_name(ast: &AST) -> Result<String, TypeErr> {
    match &ast.node {
        Node::Id { lit } => Ok(lit.clone()),
        _ => Err(TypeErr::new(&ast.pos, "Expected valid identifier"))
    }
}
