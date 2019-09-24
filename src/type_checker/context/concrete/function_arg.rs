use std::collections::HashMap;

use crate::common::position::Position;
use crate::type_checker::context::concrete::type_name::TypeName;
use crate::type_checker::context::generic::function_arg::GenericFunctionArg;
use crate::type_checker::context::generic::type_name::GenericTypeName;
use crate::type_checker::type_result::{TypeErr, TypeResult};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FunctionArg {
    pub is_py_type: bool,
    pub name: String,
    pub vararg: bool,
    pub mutable: bool,
    ty: Option<TypeName>
}

impl FunctionArg {
    pub fn try_from(
        generic_fun_arg: &GenericFunctionArg,
        generics: &HashMap<String, GenericTypeName>,
        pos: &Position
    ) -> Result<Self, TypeErr> {
        Ok(FunctionArg {
            is_py_type: generic_fun_arg.is_py_type,
            name:       generic_fun_arg.name.clone(),
            vararg:     generic_fun_arg.vararg,
            mutable:    generic_fun_arg.mutable,
            ty:         match &generic_fun_arg.ty()? {
                Some(ty) => Some(TypeName::try_from(ty, generics, pos)?),
                None => None
            }
        })
    }

    pub fn ty(&self, pos: &Position) -> TypeResult<Option<TypeName>> {
        if self.is_py_type {
            Ok(self.ty.clone())
        } else {
            Ok(Some(self.ty.clone().ok_or_else(|| TypeErr::new(pos, "Unable to get type"))?))
        }
    }
}
