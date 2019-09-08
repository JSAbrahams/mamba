use std::ops::Deref;

use crate::common::position::Position;
use crate::parser::ast::{ASTNode, ASTNodePos};
use crate::type_checker::context::class::Generic;
use crate::type_checker::context::common::try_from_id;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::context::ReturnType;
use crate::type_checker::type_result::TypeErr;

#[derive(Debug, Clone)]
pub struct Function {
    pub name:      String,
    pub pure:      bool,
    pub private:   bool,
    pub position:  Position,
    pub arguments: Vec<FunctionArg>,
    pub raises:    Vec<Generic>,
    ret_ty:        Option<TypeName>
}

#[derive(Debug, Clone)]
pub struct FunctionArg {
    pub name:     String,
    pub vararg:   bool,
    pub mutable:  bool,
    pub position: Position,
    ty:           Option<TypeName>
}

impl Function {
    /// Build a function signature from a
    /// [ASTNodePos](crate::parser::ast::ASTNodePos).
    ///
    /// # Failures
    ///
    /// If [ASTNodePos](crate::parser::ast::ASTNodePos)'s node is not the
    /// [FunDef](crate::parser::ast::ASTNode::FunDef) variant of the
    /// [ASTNode](crate::parser::ast::ASTNode).
    pub fn try_from_node_pos(node_pos: &ASTNodePos, all_pure: bool) -> Result<Function, TypeErr> {
        match &node_pos.node {
            // TODO Add type inference of body
            // TODO analyse raises/exceptions
            ASTNode::FunDef { pure, id, fun_args, ret_ty, raises, private, .. } => Ok(Function {
                name:      try_from_id(id)?,
                pure:      *pure || all_pure,
                private:   *private,
                position:  node_pos.position.clone(),
                arguments: fun_args
                    .iter()
                    .map(|arg| FunctionArg::try_from_node_pos(arg))
                    .collect::<Result<Vec<FunctionArg>, TypeErr>>()?,
                ret_ty:    match ret_ty {
                    Some(ty) => Some(TypeName::try_from_node_pos(ty.as_ref())?),
                    None => None
                },
                raises:    raises
                    .iter()
                    .map(|raise| Generic::try_from_node_pos(raise))
                    .collect::<Result<Vec<Generic>, TypeErr>>()?
            }),
            _ => Err(TypeErr::new(&node_pos.position, "Expected function definition"))
        }
    }
}

impl FunctionArg {
    pub fn try_from_node_pos(node_pos: &ASTNodePos) -> Result<FunctionArg, TypeErr> {
        match &node_pos.node {
            ASTNode::FunArg { vararg, id_maybe_type, .. } => match &id_maybe_type.node {
                ASTNode::IdType { id, mutable, _type } => Ok(FunctionArg {
                    name:     match &id.node {
                        ASTNode::Init =>
                            return Err(TypeErr::new(&id.position, "Init cannot be a function")),
                        _ => try_from_id(id.deref())?
                    },
                    vararg:   *vararg,
                    mutable:  *mutable,
                    position: node_pos.position.clone(),
                    ty:       match _type {
                        Some(_type) => Some(TypeName::try_from_node_pos(_type.deref())?),
                        None => None
                    }
                }),
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

impl ReturnType for Function {
    fn with_return_type_name(self, ty: TypeName) -> Result<Self, TypeErr> {
        if self.ret_ty.is_some() && self.ret_ty.unwrap() != ty {
            Err(TypeErr::new(&self.position, "Inferred type not equal to signature"))
        } else {
            Ok(Function {
                name:      self.name,
                pure:      self.pure,
                private:   self.private,
                position:  self.position.clone(),
                arguments: self.arguments,
                ret_ty:    Some(ty),
                raises:    self.raises
            })
        }
    }

    fn get_return_type_name(&self) -> Result<TypeName, TypeErr> {
        match &self.ret_ty {
            Some(type_name) => Ok(type_name.clone()),
            None => Err(TypeErr::new(&self.position, "Cannot infer function return type"))
        }
    }
}
