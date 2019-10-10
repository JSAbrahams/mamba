// TODO args should be literals or identifiers

use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::context::parameter::GenericParameter;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use std::convert::TryFrom;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GenericParent {
    pub is_py_type: bool,
    pub name:       String,
    pub pos:        Position,
    pub generics:   Vec<GenericParameter>,
    pub args:       Vec<TypeName>
}

impl Hash for GenericParent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.is_py_type.hash(state);
        self.name.hash(state);
        self.generics.hash(state);
        self.args.hash(state);
    }
}

impl TryFrom<&AST> for GenericParent {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> TypeResult<GenericParent> {
        match &ast.node {
            // TODO infer types of arguments passed to parent
            // TODO use arguments
            Node::Parent { id, generics, .. } => Ok(GenericParent {
                is_py_type: false,
                name:       match &id.node {
                    Node::Id { lit } => lit.clone(),
                    _ => return Err(vec![TypeErr::new(&id.pos.clone(), "Expected identifier")])
                },
                pos:        ast.pos.clone(),
                generics:   generics
                    .iter()
                    .map(GenericParameter::try_from)
                    .collect::<Result<_, _>>()?,
                args:       vec![]
            }),
            _ => Err(vec![TypeErr::new(&ast.pos.clone(), "Expected parent")])
        }
    }
}
