use crate::common::position::Position;
use crate::parser::ast::{ASTNode, ASTNodePos};
use crate::type_checker::context::common::try_from_id;
use crate::type_checker::context::field::Field;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::type_result::{TypeErr, TypeResult};

#[derive(Debug, Clone)]
pub struct Function {
    pub name:      String,
    pub pure:      bool,
    position:      Position,
    pub arguments: Vec<Field>,
    ret_ty:        Option<TypeName>,
    pub raises:    Vec<TypeName>
}

impl Function {
    pub fn try_from_node_pos(node_pos: &ASTNodePos) -> Result<Function, TypeErr> {
        match &node_pos.node {
            // TODO Add type inference of body
            // TODO analyse raises/exceptions
            ASTNode::FunDef { pure, id, fun_args, ret_ty, raises, .. } => Ok(Function {
                name:      try_from_id(id)?,
                pure:      *pure,
                position:  Position::from(node_pos),
                arguments: fun_args
                    .iter()
                    .map(|arg| Field::try_from_node_pos(arg))
                    .collect::<Result<Vec<Field>, TypeErr>>()?,
                ret_ty:    match ret_ty {
                    Some(ty) => Some(TypeName::try_from_node_pos(ty.as_ref())?),
                    None => None
                },
                raises:    raises
                    .iter()
                    .map(|raise| TypeName::try_from_node_pos(raise))
                    .collect::<Result<Vec<TypeName>, TypeErr>>()?
            }),
            _ => Err(TypeErr::new(Position::from(node_pos), "Expected function definition"))
        }
    }

    pub fn get_return_type(&self) -> Result<TypeName, TypeErr> {
        match &self.ret_ty {
            Some(type_name) => Ok(type_name.clone()),
            None => Err(TypeErr::new(self.clone().position, "Function type cannot be inferred"))
        }
    }
}

pub fn get_functions(_: &ASTNodePos) -> TypeResult<Vec<Function>> { Ok(vec![]) }
