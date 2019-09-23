use std::convert::TryFrom;
use std::ops::Deref;

use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::context::generic::field::GenericField;
use crate::type_checker::context::generic::type_name::GenericTypeName;
use crate::type_checker::type_result::TypeErr;

#[derive(Debug, Clone)]
pub struct GenericFunctionArgFieldPair {
    pub field:   Option<GenericField>,
    pub fun_arg: GenericFunctionArg
}

#[derive(Debug, Clone)]
pub struct GenericFunctionArg {
    pub name:    String,
    pub pos:     Position,
    pub vararg:  bool,
    pub mutable: bool,
    pub ty:      Option<GenericTypeName>
}

impl GenericFunctionArg {
    pub fn in_class(self, class: Option<GenericTypeName>) -> Result<Self, TypeErr> {
        if class.is_none() && self.name.as_str() == "self" {
            Err(TypeErr::new(&self.pos, "Cannot have self argument outside class"))
        } else if class.is_some() && self.name.as_str() == "self" && self.ty.is_none() {
            Ok(GenericFunctionArg {
                name:    self.name,
                pos:     self.pos,
                vararg:  self.vararg,
                mutable: self.mutable,
                ty:      class
            })
        } else {
            Ok(self)
        }
    }

    pub fn ty(&self) -> Result<GenericTypeName, TypeErr> {
        self.ty
            .clone()
            .ok_or_else(|| TypeErr::new(&self.pos.clone(), "Function argument type not given"))
    }
}

impl TryFrom<&AST> for GenericFunctionArgFieldPair {
    type Error = TypeErr;

    fn try_from(node_pos: &AST) -> Result<Self, Self::Error> {
        match &node_pos.node {
            Node::VariableDef { id_maybe_type, .. } => Ok(GenericFunctionArgFieldPair {
                field:   Some(GenericField::try_from(node_pos)?),
                fun_arg: GenericFunctionArg::try_from(&AST {
                    pos:  node_pos.pos.clone(),
                    node: Node::FunArg {
                        vararg:        false,
                        id_maybe_type: id_maybe_type.clone(),
                        default:       None
                    }
                })?
            }),
            Node::FunArg { .. } => Ok(GenericFunctionArgFieldPair {
                field:   None,
                fun_arg: GenericFunctionArg::try_from(node_pos)?
            }),
            _ => Err(TypeErr::new(&node_pos.pos, "Expected definition or function argument"))
        }
    }
}

impl TryFrom<&AST> for GenericFunctionArg {
    type Error = TypeErr;

    fn try_from(node_pos: &AST) -> Result<Self, Self::Error> {
        match &node_pos.node {
            Node::FunArg { vararg, id_maybe_type, .. } => match &id_maybe_type.node {
                Node::IdType { id, mutable, _type } => {
                    let name = argument_name(id.deref())?;
                    Ok(GenericFunctionArg {
                        name:    name.clone(),
                        vararg:  *vararg,
                        mutable: *mutable,
                        pos:     node_pos.pos.clone(),
                        ty:      match _type {
                            Some(_type) => Some(GenericTypeName::try_from(_type.deref())?),
                            None => None
                        }
                    })
                }
                _ => Err(TypeErr::new(
                    &id_maybe_type.pos,
                    "Expected function argument identifier (and type)"
                ))
            },
            _ => panic!("{:?}", node_pos.node) /* Err(TypeErr::new(&node_pos.pos, "Expected
                                                * function argument")) */
        }
    }
}

fn argument_name(ast: &AST) -> Result<String, TypeErr> {
    match &ast.node {
        Node::Id { lit } => Ok(lit.clone()),
        Node::_Self => Ok(String::from("self")),
        _ => Err(TypeErr::new(&ast.pos, "Expected identifier"))
    }
}
