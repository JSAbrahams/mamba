use crate::common::position::Position;
use crate::type_checker::context::concrete::type_name::TypeName;
use crate::type_checker::context::generic::function_arg::GenericFunctionArg;
use crate::type_checker::context::generic::type_name::GenericTypeName;
use crate::type_checker::type_result::TypeErr;
use std::collections::HashMap;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FunctionArg {
    pub name:    String,
    pub vararg:  bool,
    pub mutable: bool,
    pub ty:      TypeName
}

impl FunctionArg {
    pub fn try_from(
        generic_fun_arg: &GenericFunctionArg,
        generics: &HashMap<String, GenericTypeName>,
        pos: &Position
    ) -> Result<Self, TypeErr> {
        Ok(FunctionArg {
            name:    generic_fun_arg.name.clone(),
            vararg:  generic_fun_arg.vararg,
            mutable: generic_fun_arg.mutable,
            ty:      TypeName::try_from(&generic_fun_arg.ty()?, generics, pos)?
        })
    }
}
