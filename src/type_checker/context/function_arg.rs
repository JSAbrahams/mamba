use crate::common::position::Position;
use crate::parser::ast::{ASTNode, ASTNodePos};
use crate::type_checker::context::common::try_from_id;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::context::ReturnType;
use crate::type_checker::type_result::TypeErr;
use std::convert::TryFrom;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct FunctionArg {
    pub name:     String,
    pub vararg:   bool,
    pub mutable:  bool,
    pub position: Position,
    ty:           Option<TypeName>
}

impl FunctionArg {
    pub fn in_class(self, in_class: bool) -> Result<FunctionArg, TypeErr> {
        if !in_class && self.name.as_str() == "self" {
            Err(TypeErr::new(&self.position, "Cannot have self argument outside class"))
        } else {
            Ok(self)
        }
    }
}

impl TryFrom<&ASTNodePos> for FunctionArg {
    type Error = TypeErr;

    fn try_from(node_pos: &ASTNodePos) -> Result<Self, Self::Error> {
        match &node_pos.node {
            ASTNode::FunArg { vararg, id_maybe_type, .. } => match &id_maybe_type.node {
                ASTNode::IdType { id, mutable, _type } => {
                    let name = try_from_id(id.deref())?;
                    Ok(FunctionArg {
                        name:     name.clone(),
                        vararg:   *vararg,
                        mutable:  *mutable,
                        position: node_pos.position.clone(),
                        ty:       match _type {
                            Some(_type) => Some(TypeName::try_from_node_pos(_type.deref())?),
                            None if name.as_str() == "self" => None,
                            None =>
                                return Err(TypeErr::new(
                                    &node_pos.position,
                                    "Non self arguments must have type"
                                )),
                        }
                    })
                }
                _ => Err(TypeErr::new(
                    &id_maybe_type.position,
                    "Expected function argument identifier (and type)"
                ))
            },
            _ => Err(TypeErr::new(&node_pos.position, "Expected function argument"))
        }
    }
}

impl ReturnType for FunctionArg {
    fn with_return_type_name(self, ty: TypeName) -> Result<Self, TypeErr> {
        if self.ty.is_some() && self.ty.unwrap() != ty {
            Err(TypeErr::new(&self.position, "Cannot infer function argument type"))
        } else {
            Ok(FunctionArg {
                name:     self.name,
                vararg:   self.vararg,
                mutable:  self.mutable,
                position: self.position,
                ty:       Some(ty)
            })
        }
    }

    fn get_return_type_name(&self) -> Result<TypeName, TypeErr> {
        match &self.ty {
            Some(ty) => Ok(ty.clone()),
            None => Err(TypeErr::new(&self.position, "Cannot infer function argument type"))
        }
    }
}
