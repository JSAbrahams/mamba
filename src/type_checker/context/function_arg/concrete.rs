use crate::common::position::Position;
use crate::type_checker::context::function_arg::generic::GenericFunctionArg;
use crate::type_checker::context::type_name::concrete::TypeName;
use crate::type_checker::context::type_name::generic::GenericTypeName;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use std::collections::HashMap;
use std::convert::TryFrom;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FunctionArg {
    pub is_py_type: bool,
    pub name:       String,
    pub vararg:     bool,
    pub mutable:    bool,
    pub ty:         Option<TypeName>
}

impl FunctionArg {
    pub fn ty(&self) -> TypeResult<Option<TypeName>> {
        if self.is_py_type {
            Ok(self.ty.clone())
        } else {
            Ok(Some(self.ty.clone().ok_or_else(|| {
                vec![TypeErr::new(&self.pos.clone(), "Function argument type not given")]
            })?))
        }
    }
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
                Some(ty) => Some(TypeName::try_from((ty, generics, pos))?),
                None => None
            }
        })
    }
}
