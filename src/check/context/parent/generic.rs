use std::convert::TryFrom;
use std::hash::{Hash, Hasher};

use crate::check::context::name::{DirectName, NameUnion};
use crate::check::context::parameter::generic::GenericParameter;
use crate::check::result::{TypeErr, TypeResult};
use crate::common::position::Position;
use crate::parse::ast::{Node, AST};

#[derive(Debug, Clone, Eq)]
pub struct GenericParent {
    pub is_py_type: bool,
    pub name:       DirectName,
    pub pos:        Position,
    pub generics:   Vec<GenericParameter>,
    pub args:       Vec<NameUnion>
}

impl Hash for GenericParent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.is_py_type.hash(state);
        self.name.hash(state);
        self.generics.hash(state);
        self.args.iter().for_each(|a| a.hash(state));
    }
}

impl PartialEq for GenericParent {
    fn eq(&self, other: &Self) -> bool {
        self.is_py_type == other.is_py_type
            && self.name == other.name
            && self.generics == other.generics
            && self.args == other.args
    }
}

impl TryFrom<&AST> for GenericParent {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> TypeResult<GenericParent> {
        match &ast.node {
            // TODO infer types of arguments passed to parent
            // TODO use arguments
            Node::Parent { id, generics, .. } | Node::Type { id, generics } => Ok(GenericParent {
                is_py_type: false,
                name:       DirectName::try_from(id)?,
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