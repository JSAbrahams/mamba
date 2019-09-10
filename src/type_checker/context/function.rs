use std::convert::TryFrom;

use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::context::class::GenericParameter;
use crate::type_checker::context::common::try_from_id;
use crate::type_checker::context::function_arg::FunctionArg;
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
    pub raises:    Vec<GenericParameter>,
    ret_ty:        Option<TypeName>
}

impl Function {
    pub fn pure(self, pure: bool) -> Function {
        Function {
            name: self.name,
            pure,
            private: self.private,
            position: self.position,
            arguments: self.arguments,
            raises: self.raises,
            ret_ty: self.ret_ty
        }
    }

    pub fn in_class(self, in_class: bool) -> Result<Function, TypeErr> {
        let arguments = self
            .arguments
            .into_iter()
            .map(|arg| arg.in_class(in_class))
            .collect::<Result<Vec<FunctionArg>, TypeErr>>()?;

        Ok(Function {
            name: self.name,
            pure: self.pure,
            private: self.private,
            position: self.position,
            arguments,
            raises: self.raises,
            ret_ty: self.ret_ty
        })
    }
}

impl TryFrom<&AST> for Function {
    type Error = TypeErr;

    /// Build a function signature from a
    /// [ASTNodePos](crate::parser::ast::ASTNodePos).
    ///
    /// # Failures
    ///
    /// If [ASTNodePos](crate::parser::ast::ASTNodePos)'s node is not the
    /// [FunDef](crate::parser::ast::ASTNode::FunDef) variant of the
    /// [ASTNode](crate::parser::ast::ASTNode).
    fn try_from(node_pos: &AST) -> Result<Self, Self::Error> {
        match &node_pos.node {
            // TODO Add type inference of body
            // TODO analyse raises/exceptions
            Node::FunDef { pure, id, fun_args, ret_ty, raises, private, .. } => Ok(Function {
                name:      try_from_id(id)?,
                pure:      *pure,
                private:   *private,
                position:  node_pos.pos.clone(),
                arguments: fun_args
                    .iter()
                    .map(FunctionArg::try_from)
                    .collect::<Result<Vec<FunctionArg>, TypeErr>>()?,
                ret_ty:    match ret_ty {
                    Some(ty) => Some(TypeName::try_from(ty.as_ref())?),
                    None => None
                },
                raises:    raises
                    .iter()
                    .map(GenericParameter::try_from)
                    .collect::<Result<Vec<GenericParameter>, TypeErr>>()?
            }),
            _ => Err(TypeErr::new(&node_pos.pos, "Expected function definition"))
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
