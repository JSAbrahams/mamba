use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use std::convert::TryFrom;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct GenericField {
    pub is_py_type: bool,
    pub name:       String,
    pub pos:        Position,
    pub private:    bool,
    pub mutable:    bool,
    pub ty:         Option<TypeName>
}

impl TryFrom<&AST> for GenericField {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> TypeResult<GenericField> {
        match &ast.node {
            // TODO do something with forward
            Node::VariableDef { private, id_maybe_type, .. } => {
                let (name, mutable, ty) = match &id_maybe_type.node {
                    Node::IdType { id, mutable, _type } =>
                        (field_name(id.deref())?, *mutable, match _type {
                            Some(_ty) => Some(TypeName::try_from(_ty.deref())?),
                            None => None
                        }),
                    _ => return Err(vec![TypeErr::new(&id_maybe_type.pos, "Expected identifier")])
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
            _ => Err(vec![TypeErr::new(&ast.pos, "Expected variable")])
        }
    }
}

fn field_name(ast: &AST) -> TypeResult<String> {
    match &ast.node {
        Node::Id { lit } => Ok(lit.clone()),
        _ => Err(vec![TypeErr::new(&ast.pos, "Expected valid identifier")])
    }
}
